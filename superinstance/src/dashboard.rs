//! Dashboard - The Living Ranch UI
//! 
//! Terminal-based dashboard using ratatui to visualize:
//! - Active Species and their status
//! - Resource Usage (VRAM)
//! - Economic Counter ($ Saved)
//! - Recent activity log

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Widget, List, ListItem, canvas::Canvas},
    Frame, Terminal,
};
use tracing::{debug, info};

use crate::ranch::Ranch;
use crate::species::SpeciesType;

/// Dashboard - The Living Ranch Terminal UI
pub struct Dashboard {
    /// Reference to the Ranch
    ranch: Arc<Ranch>,
    /// Should quit flag
    should_quit: bool,
    /// Activity log
    activity_log: Vec<String>,
    /// Selected tab
    selected_tab: usize,
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new(ranch: Arc<Ranch>) -> Self {
        Self {
            ranch,
            should_quit: false,
            activity_log: vec![
                "🐔 Chickens watching the perimeter...".to_string(),
                "🐕 Collie on duty, anticipating needs...".to_string(),
                "🐄 Cattle grazing in the pasture...".to_string(),
            ],
            selected_tab: 0,
        }
    }
    
    /// Run the dashboard
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Main loop
        while !self.should_quit {
            // Draw
            terminal.draw(|f| self.draw(f))?;
            
            // Handle events
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            self.should_quit = true;
                        }
                        KeyCode::Tab => {
                            self.selected_tab = (self.selected_tab + 1) % 3;
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            // Run morning routine demo
                            self.add_activity("🌅 Running Morning Routine demo...".to_string());
                            if let Err(e) = crate::ranch::morning_routine(&self.ranch).await {
                                self.add_activity(format!("❌ Demo failed: {}", e));
                            } else {
                                self.add_activity("✅ Morning Routine complete!".to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            
            // Update activity log occasionally
            if rand::random::<f32>() < 0.05 {
                self.add_random_activity();
            }
        }
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        
        Ok(())
    }
    
    /// Draw the dashboard
    fn draw(&self, f: &mut Frame) {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Length(8),  // Species panel
                Constraint::Length(5),  // Resource usage
                Constraint::Length(3),  // Economic counter
                Constraint::Min(10),    // Activity log
                Constraint::Length(1),  // Footer
            ])
            .split(f.size());
        
        // Draw header
        self.draw_header(f, chunks[0]);
        
        // Draw species panel
        self.draw_species_panel(f, chunks[1]);
        
        // Draw resource usage
        self.draw_resources(f, chunks[2]);
        
        // Draw economic counter
        self.draw_economics(f, chunks[3]);
        
        // Draw activity log
        self.draw_activity_log(f, chunks[4]);
        
        // Draw footer
        self.draw_footer(f, chunks[5]);
    }
    
    /// Draw header
    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let day = self.ranch.get_day();
        
        let header = Paragraph::new(
            Line::from(vec![
                Span::styled("SUPERINSTANCE RANCH", Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)),
                Span::raw(" - Day "),
                Span::styled(day.to_string(), Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)),
                Span::raw(" - "),
                Span::styled("\"Not a Superintelligence, but a loyal team\"", 
                    Style::default().fg(Color::DarkGray)),
            ])
        )
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)));
        
        f.render_widget(header, area);
    }
    
    /// Draw species panel
    fn draw_species_panel(&self, f: &mut Frame, area: Rect) {
        let counts = self.ranch.get_species_counts();
        
        // Create columns for species
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
            ])
            .split(area);
        
        let species_list = [
            (SpeciesType::Cattle, "Cattle"),
            (SpeciesType::Sheep, "Sheep"),
            (SpeciesType::Duck, "Duck"),
            (SpeciesType::Goat, "Goat"),
            (SpeciesType::Hog, "Hog"),
            (SpeciesType::Chicken, "Chicken"),
            (SpeciesType::Horse, "Horse"),
        ];
        
        for (i, (species, name)) in species_list.iter().enumerate() {
            let count = counts.get(species).copied().unwrap_or(0);
            
            let text = vec![
                Line::from(Span::styled(
                    species.emoji(),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::raw(name)),
                Line::from(Span::styled(
                    format!("x{}", count),
                    Style::default().fg(Color::Cyan),
                )),
            ];
            
            let paragraph = Paragraph::new(text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)));
            
            f.render_widget(paragraph, columns[i]);
        }
    }
    
    /// Draw resource usage
    fn draw_resources(&self, f: &mut Frame, area: Rect) {
        let usage = self.ranch.get_resource_usage();
        
        // Split into two rows
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        
        // VRAM gauge
        let vram_label = format!("VRAM: {:.1}%", usage.vram_used_percent());
        let vram_gauge = Gauge::default()
            .block(Block::default()
                .title(" GPU Memory ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(usage.vram_used_percent() as u16)
            .label(vram_label);
        
        f.render_widget(vram_gauge, rows[0]);
        
        // Task stats
        let task_text = format!(
            "Tasks: {} completed | {} failed | {} pending | {} adapters active",
            usage.completed_tasks,
            usage.failed_tasks,
            usage.pending_tasks,
            usage.active_adapters,
        );
        
        let task_paragraph = Paragraph::new(task_text)
            .block(Block::default()
                .title(" Activity ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)));
        
        f.render_widget(task_paragraph, rows[1]);
    }
    
    /// Draw economic counter
    fn draw_economics(&self, f: &mut Frame, area: Rect) {
        let dollars = self.ranch.get_dollars_saved();
        
        let text = format!("💰 Cloud API Savings: ${:.2}", dollars);
        
        let paragraph = Paragraph::new(
            Line::from(vec![
                Span::styled(text, Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)),
                Span::raw("  |  "),
                Span::styled(
                    "Local inference: 100% | Cloud calls: 0",
                    Style::default().fg(Color::Green)
                ),
            ])
        )
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));
        
        f.render_widget(paragraph, area);
    }
    
    /// Draw activity log
    fn draw_activity_log(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.activity_log
            .iter()
            .rev()
            .take(area.height as usize)
            .map(|msg| {
                ListItem::new(Line::from(Span::raw(msg)))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .title(" Activity Log ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)));
        
        f.render_widget(list, area);
    }
    
    /// Draw footer
    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let footer = Paragraph::new(
            Line::from(vec![
                Span::styled("[D]", Style::default().fg(Color::Cyan)),
                Span::raw(" Demo  "),
                Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
                Span::raw(" Switch Tab  "),
                Span::styled("[Q]", Style::default().fg(Color::Cyan)),
                Span::raw(" Quit"),
            ])
        );
        
        f.render_widget(footer, area);
    }
    
    /// Add activity to log
    fn add_activity(&mut self, msg: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        self.activity_log.push(format!("[{}] {}", timestamp, msg));
        
        // Keep log size reasonable
        if self.activity_log.len() > 100 {
            self.activity_log.remove(0);
        }
    }
    
    /// Add random activity for demo
    fn add_random_activity(&mut self) {
        let activities = [
            "🐔 Chicken detected motion at perimeter",
            "🐕 Collie anticipating user intent...",
            "🦆 Duck fetched calendar data successfully",
            "🐑 Sheep flock reached consensus on email classification",
            "🐐 Goat navigated complex log file",
            "🐄 Cattle completed deep reasoning task",
            "🐗 Hog responded in <10ms to sensor event",
            "🐎 Horse finished ETL pipeline",
            "✨ Reflex cache hit - instant response!",
            "🔮 Shadow Pup predicted next action correctly",
        ];
        
        let idx = rand::random::<usize>() % activities.len();
        self.add_activity(activities[idx].to_string());
    }
}

/// Widget for displaying a single species card
pub struct SpeciesCard {
    pub species: SpeciesType,
    pub count: usize,
    pub status: &'static str,
}

impl Widget for SpeciesCard {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let text = vec![
            Line::from(Span::styled(
                self.species.emoji(),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::raw(self.status)),
        ];
        
        Paragraph::new(text).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dashboard_creation() {
        // Dashboard creation requires a Ranch
        // This test verifies the module compiles
    }
}
