'use client'

/**
 * SuperInstance Dashboard - Main Frontend Interface
 * 
 * This is the web dashboard for the SuperInstance Ranch ecosystem. It provides:
 * - Real-time monitoring of all species agents
 * - GPU/CPU/RAM resource visualization
 * - Live activity log with WebSocket updates
 * - Night School trigger controls
 * - Open Genomics breed.md preview
 * - Memory Pasture statistics
 * - Channel connector status
 * 
 * Architecture:
 * - Uses Astro-inspired "islands" approach with minimal JS
 * - Connects to Rust backend via WebSocket for real-time updates
 * - Simulates data in development mode for demonstration
 * 
 * @module SuperInstanceDashboard
 * @see {@link https://github.com/SuperInstance/superinstance}
 */

import { useState, useEffect, useCallback } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { 
  Activity, Cpu, HardDrive, Wifi, Zap, Moon, Sun, Terminal,
  Bot, MessageSquare, Database, GitBranch, Clock, TrendingUp,
  AlertCircle, CheckCircle2, Loader2, Pause, Play, Settings,
  ChevronRight, Sparkles, Brain, Heart, Target
} from 'lucide-react'

/**
 * Species interface matching Rust backend Species struct
 * @typedef {Object} Species
 */
interface Species {
  /** Unique identifier for this species instance */
  id: string
  /** Display name (e.g., "Cattle", "Sheep") */
  name: string
  /** Emoji representation for UI */
  emoji: string
  /** Current operational status */
  status: 'active' | 'idle' | 'evolving'
  /** Number of completed tasks */
  tasks_completed: number
  /** Fitness score (0.0-1.0) for evolution */
  fitness: number
  /** VRAM usage in megabytes */
  vram_mb: number
}

/**
 * RanchState interface matching Rust backend Ranch state
 * @typedef {Object} RanchState
 */
interface RanchState {
  /** Current day count (increments after each Night School) */
  day: number
  /** Array of all species agents */
  species: Species[]
  /** Current VRAM usage in GB */
  vram_used_gb: number
  /** Total VRAM available in GB */
  vram_total_gb: number
  /** CPU utilization percentage */
  cpu_percent: number
  /** RAM usage in GB */
  ram_gb: number
  /** Dollars saved by using local vs cloud */
  savings: number
  /** Number of cloud API calls avoided */
  cloud_calls_avoided: number
  /** Current tokens per second generation rate */
  tokens_per_sec: number
  /** First token latency in milliseconds */
  first_token_ms: number
  /** Core binary size in MB */
  binary_mb: number
}

/**
 * Activity log entry
 * @typedef {Object} Activity
 */
interface Activity {
  /** Timestamp of the activity */
  time: string
  /** Species emoji that generated the activity */
  emoji: string
  /** Activity message */
  message: string
}

/**
 * Species configuration for the 7 core agent types
 * Maps species IDs to their display properties
 */
const SPECIES_CONFIG = [
  { id: 'cattle', name: 'Cattle', emoji: '🐄', role: 'Heavy Reasoning' },
  { id: 'sheep', name: 'Sheep', emoji: '🐑', role: 'Consensus Voting' },
  { id: 'duck', name: 'Duck', emoji: '🦆', role: 'Network API' },
  { id: 'goat', name: 'Goat', emoji: '🐐', role: 'Debug Navigate' },
  { id: 'hog', name: 'Hog', emoji: '🐗', role: 'Hardware GPIO' },
  { id: 'chicken', name: 'Chicken', emoji: '🐔', role: 'Monitor Watchdog' },
  { id: 'horse', name: 'Horse', emoji: '🐎', role: 'Cloud Fallback' },
] as const

/**
 * SuperInstance Dashboard Component
 * 
 * Main dashboard for monitoring and controlling the AI Ranch.
 * Displays real-time state of all agents, resources, and activities.
 * 
 * @returns {JSX.Element} The dashboard UI
 */
export default function SuperInstanceDashboard() {
  const [ranchState, setRanchState] = useState<RanchState>({
    day: 14,
    species: SPECIES_CONFIG.map((s, i) => ({
      id: s.id,
      name: s.name,
      emoji: s.emoji,
      status: i < 6 ? 'active' : 'idle',
      tasks_completed: Math.floor(Math.random() * 500) + 50,
      fitness: 0.6 + Math.random() * 0.35,
      vram_mb: [500, 100, 150, 150, 10, 5, 0][i]
    })),
    vram_used_gb: 5.4,
    vram_total_gb: 8.0,
    cpu_percent: 28,
    ram_gb: 3.2,
    savings: 127.50,
    cloud_calls_avoided: 255,
    tokens_per_sec: 20.3,
    first_token_ms: 4.5,
    binary_mb: 4.2
  })

  const [activities, setActivities] = useState<Activity[]>([
    { time: '10:23', emoji: '🐔', message: 'Motion detected (perimeter)' },
    { time: '10:22', emoji: '🦆', message: 'Fetched calendar data' },
    { time: '10:20', emoji: '🐑', message: 'Consensus: 4 spam, 12 good' },
    { time: '10:18', emoji: '🐐', message: 'Navigated /var/log/syslog' },
    { time: '10:15', emoji: '🐄', message: 'Briefing generated' },
    { time: '10:10', emoji: '🐔', message: 'Heartbeat OK' },
  ])

  const [nightSchoolActive, setNightSchoolActive] = useState(false)
  const [nightSchoolPhase, setNightSchoolPhase] = useState('')
  const [connected, setConnected] = useState(true)

  // Simulate WebSocket connection to Rust backend
  useEffect(() => {
    const interval = setInterval(() => {
      // Simulate real-time updates from Rust Collie
      setRanchState(prev => ({
        ...prev,
        cpu_percent: Math.max(15, Math.min(85, prev.cpu_percent + (Math.random() - 0.5) * 10)),
        vram_used_gb: Math.max(4.5, Math.min(6.0, prev.vram_used_gb + (Math.random() - 0.5) * 0.2)),
        tokens_per_sec: 18 + Math.random() * 4,
        savings: prev.savings + (Math.random() > 0.7 ? 0.05 : 0),
        cloud_calls_avoided: prev.cloud_calls_avoided + (Math.random() > 0.6 ? 1 : 0),
      }))

      // Add new activity occasionally
      if (Math.random() > 0.7) {
        const randomSpecies = SPECIES_CONFIG[Math.floor(Math.random() * SPECIES_CONFIG.length)]
        const messages = [
          'Task completed successfully',
          'Processing request...',
          'Memory retrieved from pasture',
          'LoRA adapter swapped',
          'Intent routed',
          'Response generated',
        ]
        const now = new Date()
        const time = `${now.getHours()}:${String(now.getMinutes()).padStart(2, '0')}`
        
        setActivities(prev => [{
          time,
          emoji: randomSpecies.emoji,
          message: messages[Math.floor(Math.random() * messages.length)]
        }, ...prev.slice(0, 9)])
      }
    }, 2000)

    return () => clearInterval(interval)
  }, [])

  const triggerNightSchool = useCallback(() => {
    setNightSchoolActive(true)
    const phases = ['Evaluating agents...', 'Culling underperformers...', 'Breeding champions...', 'Distilling knowledge...', 'Promoting offspring...']
    let phaseIndex = 0
    
    const interval = setInterval(() => {
      if (phaseIndex < phases.length) {
        setNightSchoolPhase(phases[phaseIndex])
        phaseIndex++
      } else {
        clearInterval(interval)
        setNightSchoolActive(false)
        setNightSchoolPhase('')
        setRanchState(prev => ({ ...prev, day: prev.day + 1 }))
      }
    }, 2000)
  }, [])

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950 text-slate-100">
      {/* Header */}
      <header className="border-b border-slate-800/50 bg-slate-900/50 backdrop-blur-xl sticky top-0 z-50">
        <div className="container mx-auto px-4 py-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="text-3xl">🐄</div>
              <div>
                <h1 className="text-xl font-bold tracking-tight">SuperInstance Ranch</h1>
                <p className="text-xs text-slate-400">Day {ranchState.day} &mdash; &quot;Not a Superintelligence...&quot;</p>
              </div>
            </div>
            
            <div className="flex items-center gap-4">
              <Badge 
                variant="outline" 
                className="gap-1.5 border-emerald-500/50 text-emerald-400"
                role="status"
                aria-label={connected ? 'Connection status: online' : 'Connection status: offline'}
              >
                <span 
                  className={`w-2 h-2 rounded-full ${connected ? 'bg-emerald-400 animate-pulse' : 'bg-red-400'}`} 
                  aria-hidden="true"
                />
                {connected ? 'Collie Online' : 'Disconnected'}
              </Badge>
              
              <Badge variant="outline" className="gap-1.5 border-amber-500/50 text-amber-400">
                <Zap className="w-3 h-3" />
                {ranchState.binary_mb} MB Core
              </Badge>
              
              <Button size="sm" variant="outline" className="gap-1.5" onClick={triggerNightSchool} disabled={nightSchoolActive}>
                {nightSchoolActive ? (
                  <>
                    <Loader2 className="w-4 h-4 animate-spin" />
                    {nightSchoolPhase}
                  </>
                ) : (
                  <>
                    <Moon className="w-4 h-4" />
                    Night School
                  </>
                )}
              </Button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-4 py-6">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          
          {/* Left Column - Species */}
          <div className="lg:col-span-3 space-y-4">
            <Card className="bg-slate-900/50 border-slate-800/50 backdrop-blur">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                  <Bot className="w-4 h-4 text-violet-400" />
                  Species Panel
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                {ranchState.species.map((species) => (
                  <button
                    key={species.id}
                    className="flex items-center justify-between p-2 rounded-lg bg-slate-800/30 hover:bg-slate-800/50 transition-colors cursor-pointer group w-full text-left"
                    onClick={() => {/* TODO: Open species details */}}
                    aria-label={`${species.name}: ${species.tasks_completed} tasks completed, status: ${species.status}`}
                  >
                    <div className="flex items-center gap-2">
                      <span className="text-xl" aria-hidden="true">{species.emoji}</span>
                      <div>
                        <div className="text-sm font-medium">{species.name}</div>
                        <div className="text-xs text-slate-500">{species.tasks_completed} tasks</div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge 
                        variant="outline" 
                        className={`text-xs ${
                          species.status === 'active' 
                            ? 'border-emerald-500/50 text-emerald-400' 
                            : species.status === 'evolving'
                            ? 'border-amber-500/50 text-amber-400'
                            : 'border-slate-600 text-slate-400'
                        }`}
                      >
                        {species.status}
                      </Badge>
                      <ChevronRight className="w-4 h-4 text-slate-600 group-hover:text-slate-400 transition-colors" aria-hidden="true" />
                    </div>
                  </button>
                ))}
              </CardContent>
            </Card>

            {/* Quick Actions */}
            <Card className="bg-slate-900/50 border-slate-800/50 backdrop-blur">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Quick Actions</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <Button variant="outline" className="w-full justify-start gap-2 bg-slate-800/30" size="sm">
                  <Terminal className="w-4 h-4" />
                  Open TUI Dashboard
                </Button>
                <Button variant="outline" className="w-full justify-start gap-2 bg-slate-800/30" size="sm">
                  <GitBranch className="w-4 h-4" />
                  Create New Breed
                </Button>
                <Button variant="outline" className="w-full justify-start gap-2 bg-slate-800/30" size="sm">
                  <Database className="w-4 h-4" />
                  Memory Pasture
                </Button>
              </CardContent>
            </Card>
          </div>

          {/* Center Column - Dashboard */}
          <div className="lg:col-span-6 space-y-4">
            {/* Performance Metrics */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              <Card className="bg-gradient-to-br from-violet-900/30 to-slate-900/50 border-violet-800/30">
                <CardContent className="pt-4">
                  <div className="flex items-center gap-2 mb-1">
                    <Zap className="w-4 h-4 text-violet-400" />
                    <span className="text-xs text-slate-400">Tokens/sec</span>
                  </div>
                  <div className="text-2xl font-bold">{ranchState.tokens_per_sec.toFixed(1)}</div>
                  <div className="text-xs text-emerald-400">+98% vs llama.cpp</div>
                </CardContent>
              </Card>
              
              <Card className="bg-gradient-to-br from-emerald-900/30 to-slate-900/50 border-emerald-800/30">
                <CardContent className="pt-4">
                  <div className="flex items-center gap-2 mb-1">
                    <Clock className="w-4 h-4 text-emerald-400" />
                    <span className="text-xs text-slate-400">First Token</span>
                  </div>
                  <div className="text-2xl font-bold">{ranchState.first_token_ms}ms</div>
                  <div className="text-xs text-emerald-400">TensorRT-LLM</div>
                </CardContent>
              </Card>
              
              <Card className="bg-gradient-to-br from-amber-900/30 to-slate-900/50 border-amber-800/30">
                <CardContent className="pt-4">
                  <div className="flex items-center gap-2 mb-1">
                    <TrendingUp className="w-4 h-4 text-amber-400" />
                    <span className="text-xs text-slate-400">Savings</span>
                  </div>
                  <div className="text-2xl font-bold">${ranchState.savings.toFixed(2)}</div>
                  <div className="text-xs text-emerald-400">{ranchState.cloud_calls_avoided} avoided</div>
                </CardContent>
              </Card>
              
              <Card className="bg-gradient-to-br from-rose-900/30 to-slate-900/50 border-rose-800/30">
                <CardContent className="pt-4">
                  <div className="flex items-center gap-2 mb-1">
                    <Heart className="w-4 h-4 text-rose-400" />
                    <span className="text-xs text-slate-400">Fitness</span>
                  </div>
                  <div className="text-2xl font-bold">
                    {(ranchState.species.reduce((a, s) => a + s.fitness, 0) / ranchState.species.length * 100).toFixed(0)}%
                  </div>
                  <div className="text-xs text-emerald-400">Avg population</div>
                </CardContent>
              </Card>
            </div>

            {/* Main Dashboard Tabs */}
            <Tabs defaultValue="activity" className="w-full">
              <TabsList className="grid w-full grid-cols-4 bg-slate-800/50">
                <TabsTrigger value="activity" className="text-xs">Activity</TabsTrigger>
                <TabsTrigger value="genetics" className="text-xs">Genetics</TabsTrigger>
                <TabsTrigger value="memory" className="text-xs">Memory</TabsTrigger>
                <TabsTrigger value="channels" className="text-xs">Channels</TabsTrigger>
              </TabsList>
              
              <TabsContent value="activity" className="mt-4">
                <Card className="bg-slate-900/50 border-slate-800/50 backdrop-blur">
                  <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                      <Activity className="w-4 h-4 text-emerald-400" />
                      Live Activity Log
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <ScrollArea className="h-64">
                      <div className="space-y-1">
                        {activities.map((activity, i) => (
                          <div 
                            key={i}
                            className="flex items-center gap-3 p-2 rounded bg-slate-800/20 hover:bg-slate-800/40 transition-colors text-sm"
                          >
                            <span className="text-xs text-slate-500 w-12">{activity.time}</span>
                            <span className="text-lg">{activity.emoji}</span>
                            <span className="text-slate-300">{activity.message}</span>
                          </div>
                        ))}
                      </div>
                    </ScrollArea>
                  </CardContent>
                </Card>
              </TabsContent>
              
              <TabsContent value="genetics" className="mt-4">
                <Card className="bg-slate-900/50 border-slate-800/50 backdrop-blur">
                  <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                      <Brain className="w-4 h-4 text-violet-400" />
                      Open Genomics
                    </CardTitle>
                    <CardDescription>Edit DNA in Markdown - changes apply instantly</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="bg-slate-800/50 rounded-lg p-4 font-mono text-xs">
                      <pre className="text-slate-300">
{`# 🐄 Breed: Email-Cow-v1

## 🧬 Genetic Composition
| Gene Trait    | Weight | Description              |
|--------------|--------|--------------------------|
| polite_tone  | 0.8    | Strong formal style      |
| json_output  | 0.1    | Light structure enforce  |
| concise      | 0.5    | Medium brevity influence |

## 🧠 System Prompt
\`\`\`
You are an Email Triage Specialist.
Prioritize emails from "Boss".
Always be concise.
\`\`\``}
                      </pre>
                    </div>
                    <Button className="w-full mt-3" variant="outline">
                      <Sparkles className="w-4 h-4 mr-2" />
                      Open breed.md Editor
                    </Button>
                  </CardContent>
                </Card>
              </TabsContent>
              
              <TabsContent value="memory" className="mt-4">
                <Card className="bg-slate-900/50 border-slate-800/50 backdrop-blur">
                  <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                      <Database className="w-4 h-4 text-cyan-400" />
                      Memory Pasture
                    </CardTitle>
                    <CardDescription>CRDT-powered persistent memory across reboots</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-3 gap-3">
                      <div className="bg-slate-800/50 rounded-lg p-3 text-center">
                        <div className="text-2xl font-bold text-cyan-400">1.2GB</div>
                        <div className="text-xs text-slate-400">Total Memory</div>
                      </div>
                      <div className="bg-slate-800/50 rounded-lg p-3 text-center">
                        <div className="text-2xl font-bold text-emerald-400">47,832</div>
                        <div className="text-xs text-slate-400">Embeddings</div>
                      </div>
                      <div className="bg-slate-800/50 rounded-lg p-3 text-center">
                        <div className="text-2xl font-bold text-violet-400">12ms</div>
                        <div className="text-xs text-slate-400">RAG Latency</div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>
              
              <TabsContent value="channels" className="mt-4">
                <Card className="bg-slate-900/50 border-slate-800/50 backdrop-blur">
                  <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium flex items-center gap-2">
                      <MessageSquare className="w-4 h-4 text-indigo-400" />
                      Channel Connectors
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-2 gap-3">
                      {[
                        { name: 'Discord', emoji: '🎮', status: 'connected', messages: 1284 },
                        { name: 'Telegram', emoji: '📱', status: 'connected', messages: 856 },
                        { name: 'Slack', emoji: '💼', status: 'idle', messages: 0 },
                        { name: 'Webhook', emoji: '🔗', status: 'active', messages: 3421 },
                      ].map((channel) => (
                        <div key={channel.name} className="bg-slate-800/50 rounded-lg p-3 flex items-center justify-between">
                          <div className="flex items-center gap-2">
                            <span className="text-xl">{channel.emoji}</span>
                            <div>
                              <div className="text-sm font-medium">{channel.name}</div>
                              <div className="text-xs text-slate-400">{channel.messages} msgs</div>
                            </div>
                          </div>
                          <Badge variant="outline" className={`text-xs ${
                            channel.status === 'connected' 
                              ? 'border-emerald-500/50 text-emerald-400' 
                              : 'border-slate-600 text-slate-400'
                          }`}>
                            {channel.status}
                          </Badge>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              </TabsContent>
            </Tabs>
          </div>

          {/* Right Column - Resources */}
          <div className="lg:col-span-3 space-y-4">
            {/* GPU Memory */}
            <Card className="bg-slate-900/50 border-slate-800/50 backdrop-blur">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                  <Cpu className="w-4 h-4 text-emerald-400" />
                  GPU Memory
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div>
                    <div className="flex justify-between text-xs mb-1">
                      <span className="text-slate-400">VRAM Usage</span>
                      <span className="text-emerald-400">{ranchState.vram_used_gb.toFixed(1)} / {ranchState.vram_total_gb} GB</span>
                    </div>
                    <Progress 
                      value={(ranchState.vram_used_gb / ranchState.vram_total_gb) * 100} 
                      className="h-2 bg-slate-800"
                    />
                  </div>
                  
                  <div className="grid grid-cols-2 gap-2 text-xs">
                    <div className="bg-slate-800/30 rounded p-2">
                      <div className="text-slate-400">Base Model</div>
                      <div className="text-slate-200 font-medium">2.5 GB</div>
                    </div>
                    <div className="bg-slate-800/30 rounded p-2">
                      <div className="text-slate-400">LoRA Pool</div>
                      <div className="text-slate-200 font-medium">1.0 GB</div>
                    </div>
                    <div className="bg-slate-800/30 rounded p-2">
                      <div className="text-slate-400">KV Cache</div>
                      <div className="text-slate-200 font-medium">1.0 GB</div>
                    </div>
                    <div className="bg-slate-800/30 rounded p-2">
                      <div className="text-slate-400">Buffer</div>
                      <div className="text-slate-200 font-medium">0.9 GB</div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* System Resources */}
            <Card className="bg-slate-900/50 border-slate-800/50 backdrop-blur">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                  <HardDrive className="w-4 h-4 text-cyan-400" />
                  System Resources
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div>
                  <div className="flex justify-between text-xs mb-1">
                    <span className="text-slate-400">CPU</span>
                    <span className="text-slate-200">{ranchState.cpu_percent.toFixed(0)}%</span>
                  </div>
                  <Progress value={ranchState.cpu_percent} className="h-1.5 bg-slate-800" />
                </div>
                <div>
                  <div className="flex justify-between text-xs mb-1">
                    <span className="text-slate-400">RAM</span>
                    <span className="text-slate-200">{ranchState.ram_gb.toFixed(1)} GB</span>
                  </div>
                  <Progress value={(ranchState.ram_gb / 8) * 100} className="h-1.5 bg-slate-800" />
                </div>
              </CardContent>
            </Card>

            {/* Night School Status */}
            <Card className="bg-gradient-to-br from-indigo-900/30 to-slate-900/50 border-indigo-800/30">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium flex items-center gap-2">
                  <Moon className="w-4 h-4 text-indigo-400" />
                  Night School
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2 text-xs">
                  <div className="flex items-center gap-2 text-slate-300">
                    <CheckCircle2 className="w-3 h-3 text-emerald-400" />
                    Last run: Today 02:00
                  </div>
                  <div className="flex items-center gap-2 text-slate-300">
                    <Target className="w-3 h-3 text-violet-400" />
                    2 breeds culled, 3 created
                  </div>
                  <div className="flex items-center gap-2 text-slate-300">
                    <TrendingUp className="w-3 h-3 text-amber-400" />
                    Avg fitness: +4.2%
                  </div>
                </div>
                <div className="mt-3 pt-3 border-t border-slate-700/50 text-xs text-slate-400">
                  Next run: Tomorrow 02:00
                </div>
              </CardContent>
            </Card>

            {/* Architecture Badge */}
            <Card className="bg-gradient-to-br from-emerald-900/20 to-slate-900/50 border-emerald-800/30">
              <CardContent className="pt-4 text-center">
                <div className="text-3xl mb-2">🌱</div>
                <div className="text-sm font-medium text-emerald-400">Just-So Architecture</div>
                <div className="text-xs text-slate-400 mt-1">
                  {ranchState.binary_mb} MB core • Zero Node runtime<br/>
                  Astro UI • Axum backend • TensorRT-LLM
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-slate-800/50 bg-slate-900/30 mt-8 py-4">
        <div className="container mx-auto px-4 text-center text-xs text-slate-500">
          <p>&quot;Don&apos;t rent an AI brain. Breed a Ranch.&quot; • SuperInstance v0.1.0 • Jetson Orin Nano 8GB</p>
        </div>
      </footer>
    </div>
  )
}
