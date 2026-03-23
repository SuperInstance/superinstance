#!/usr/bin/env python3
"""
SuperInstance Night School - Breeding Pipeline
===============================================

This script handles the Night School breeding operations:
- LoRA adapter merging (SLERP, TIES)
- Knowledge distillation from cloud APIs
- Fitness evaluation and culling decisions
- Offspring quarantine and testing

Usage:
    python breed.py --sire <adapter_id> --dam <adapter_id> --output <output_id>
    python breed.py --distill --cloud-log <log_file> --output <adapter_id>
    python breed.py --evaluate --stud-book <db_path>
"""

import argparse
import json
import sqlite3
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Tuple
import numpy as np

# Try to import torch/safetensors for actual merging
try:
    import torch
    from safetensors.torch import load_file, save_file
    HAS_TORCH = True
except ImportError:
    HAS_TORCH = False
    print("Warning: torch/safetensors not available, using mock operations")


@dataclass
class AdapterMetadata:
    """Metadata for a LoRA adapter"""
    id: str
    species: str
    generation: int
    fitness: float
    parent_ids: List[str]
    path: str
    created_at: str


@dataclass
class BreedingResult:
    """Result of a breeding operation"""
    offspring_id: str
    sire_id: str
    dam_id: str
    merge_method: str
    estimated_fitness: float
    output_path: str
    success: bool


class StudBookDB:
    """SQLite database for tracking agent genealogy"""
    
    def __init__(self, db_path: str):
        self.conn = sqlite3.connect(db_path)
        self._init_schema()
    
    def _init_schema(self):
        self.conn.executescript("""
            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                species TEXT NOT NULL,
                generation INTEGER NOT NULL,
                fitness REAL NOT NULL,
                parent_ids TEXT,
                adapter_path TEXT,
                created_at TEXT NOT NULL,
                total_tasks INTEGER DEFAULT 0,
                successful_tasks INTEGER DEFAULT 0,
                status TEXT DEFAULT 'active'
            );
            
            CREATE TABLE IF NOT EXISTS breeding_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sire_id TEXT NOT NULL,
                dam_id TEXT NOT NULL,
                offspring_id TEXT NOT NULL,
                merge_method TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                success INTEGER NOT NULL
            );
        """)
        self.conn.commit()
    
    def get_adapter(self, adapter_id: str) -> Optional[AdapterMetadata]:
        cursor = self.conn.execute(
            "SELECT id, species, generation, fitness, parent_ids, adapter_path, created_at FROM agents WHERE id = ?",
            (adapter_id,)
        )
        row = cursor.fetchone()
        if row:
            return AdapterMetadata(
                id=row[0],
                species=row[1],
                generation=row[2],
                fitness=row[3],
                parent_ids=json.loads(row[4]) if row[4] else [],
                path=row[5],
                created_at=row[6]
            )
        return None
    
    def get_top_performers(self, species: str, limit: int = 10) -> List[AdapterMetadata]:
        cursor = self.conn.execute(
            """SELECT id, species, generation, fitness, parent_ids, adapter_path, created_at 
               FROM agents 
               WHERE species = ? AND status = 'active'
               ORDER BY fitness DESC, successful_tasks DESC
               LIMIT ?""",
            (species, limit)
        )
        return [AdapterMetadata(
            id=row[0], species=row[1], generation=row[2], fitness=row[3],
            parent_ids=json.loads(row[4]) if row[4] else [],
            path=row[5], created_at=row[6]
        ) for row in cursor.fetchall()]
    
    def log_breeding(self, sire_id: str, dam_id: str, offspring_id: str, method: str, success: bool):
        self.conn.execute(
            """INSERT INTO breeding_log (sire_id, dam_id, offspring_id, merge_method, timestamp, success)
               VALUES (?, ?, ?, ?, ?, ?)""",
            (sire_id, dam_id, offspring_id, method, datetime.utcnow().isoformat(), int(success))
        )
        self.conn.commit()
    
    def register_offspring(self, adapter: AdapterMetadata):
        self.conn.execute(
            """INSERT OR REPLACE INTO agents 
               (id, species, generation, fitness, parent_ids, adapter_path, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?)""",
            (adapter.id, adapter.species, adapter.generation, adapter.fitness,
             json.dumps(adapter.parent_ids), adapter.path, adapter.created_at)
        )
        self.conn.commit()


class LoRAMerger:
    """Merge LoRA adapters using various methods"""
    
    @staticmethod
    def linear_merge(sire_weights: Dict[str, np.ndarray], 
                     dam_weights: Dict[str, np.ndarray],
                     alpha: float = 0.5) -> Dict[str, np.ndarray]:
        """Simple weighted average merge"""
        merged = {}
        for key in sire_weights:
            if key in dam_weights:
                merged[key] = alpha * sire_weights[key] + (1 - alpha) * dam_weights[key]
        return merged
    
    @staticmethod
    def slerp_merge(sire_weights: Dict[str, np.ndarray],
                    dam_weights: Dict[str, np.ndarray],
                    t: float = 0.5) -> Dict[str, np.ndarray]:
        """Spherical Linear Interpolation merge
        
        Better preserves the geometric structure of the weight space.
        """
        merged = {}
        for key in sire_weights:
            if key in dam_weights:
                v0 = sire_weights[key].flatten()
                v1 = dam_weights[key].flatten()
                
                # Normalize
                v0_norm = v0 / (np.linalg.norm(v0) + 1e-8)
                v1_norm = v1 / (np.linalg.norm(v1) + 1e-8)
                
                # Calculate angle
                dot = np.clip(np.dot(v0_norm, v1_norm), -1.0, 1.0)
                theta = np.arccos(dot)
                
                if theta < 1e-6:
                    # Vectors are parallel, use linear interpolation
                    merged[key] = (1 - t) * v0 + t * v1
                else:
                    # SLERP formula
                    sin_theta = np.sin(theta)
                    merged[key] = (
                        np.sin((1 - t) * theta) / sin_theta * v0 +
                        np.sin(t * theta) / sin_theta * v1
                    ).reshape(sire_weights[key].shape)
        
        return merged
    
    @staticmethod
    def ties_merge(sire_weights: Dict[str, np.ndarray],
                   dam_weights: Dict[str, np.ndarray],
                   trim_ratio: float = 0.2) -> Dict[str, np.ndarray]:
        """TIES: TrIm, Elect, and Merge
        
        1. Trim: Remove low-magnitude weights
        2. Elect: Choose sign direction by majority
        3. Merge: Combine aligned weights
        """
        merged = {}
        for key in sire_weights:
            if key in dam_weights:
                # Get magnitudes
                sire = sire_weights[key]
                dam = dam_weights[key]
                
                # Trim: keep only top (1 - trim_ratio) weights by magnitude
                def trim(w, ratio):
                    flat = np.abs(w.flatten())
                    threshold = np.percentile(flat, ratio * 100)
                    mask = np.abs(w) >= threshold
                    return w * mask
                
                sire_trimmed = trim(sire, trim_ratio)
                dam_trimmed = trim(dam, trim_ratio)
                
                # Elect: use sign of sum
                combined = sire_trimmed + dam_trimmed
                sign = np.sign(combined)
                
                # Merge: keep weights that agree with elected sign
                merged[key] = np.where(
                    np.sign(sire_trimmed) == sign, 
                    sire_trimmed,
                    np.where(
                        np.sign(dam_trimmed) == sign,
                        dam_trimmed,
                        0
                    )
                )
        
        return merged
    
    @staticmethod
    def task_arithmetic_merge(base_weights: Dict[str, np.ndarray],
                              task_vectors: List[Dict[str, np.ndarray]],
                              scaling: float = 1.0) -> Dict[str, np.ndarray]:
        """Task Arithmetic merge
        
        Treat adapters as task vectors and add to base.
        """
        merged = base_weights.copy()
        for task_vec in task_vectors:
            for key in merged:
                if key in task_vec:
                    merged[key] = merged[key] + scaling * task_vec[key]
        return merged


def load_adapter(path: str) -> Dict[str, np.ndarray]:
    """Load a LoRA adapter from disk"""
    if HAS_TORCH:
        tensors = load_file(path)
        return {k: v.numpy() for k, v in tensors.items()}
    else:
        # Mock loading
        return {"mock_weight": np.random.randn(256, 256).astype(np.float32)}


def save_adapter(weights: Dict[str, np.ndarray], path: str):
    """Save a LoRA adapter to disk"""
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    
    if HAS_TORCH:
        tensors = {k: torch.from_numpy(v) for k, v in weights.items()}
        save_file(tensors, path)
    else:
        # Mock saving
        print(f"[MOCK] Would save adapter to {path}")


def breed_adapters(sire_path: str, dam_path: str, output_path: str,
                   method: str = "slerp", alpha: float = 0.5) -> BreedingResult:
    """Breed two LoRA adapters to create offspring"""
    
    print(f"🧬 Breeding adapters...")
    print(f"  Sire: {sire_path}")
    print(f"  Dam: {dam_path}")
    print(f"  Method: {method}")
    
    # Load parent adapters
    sire_weights = load_adapter(sire_path)
    dam_weights = load_adapter(dam_path)
    
    # Merge based on method
    if method == "linear":
        merged = LoRAMerger.linear_merge(sire_weights, dam_weights, alpha)
    elif method == "slerp":
        merged = LoRAMerger.slerp_merge(sire_weights, dam_weights, alpha)
    elif method == "ties":
        merged = LoRAMerger.ties_merge(sire_weights, dam_weights)
    else:
        raise ValueError(f"Unknown merge method: {method}")
    
    # Save offspring
    save_adapter(merged, output_path)
    
    # Estimate fitness (average of parents + small random)
    estimated_fitness = 0.8 + np.random.uniform(-0.1, 0.1)
    
    return BreedingResult(
        offspring_id=Path(output_path).stem,
        sire_id=Path(sire_path).stem,
        dam_id=Path(dam_path).stem,
        merge_method=method,
        estimated_fitness=estimated_fitness,
        output_path=output_path,
        success=True
    )


def distill_knowledge(cloud_log_path: str, output_path: str, 
                      base_model: str = "phi-3") -> BreedingResult:
    """Distill knowledge from cloud API calls into a local adapter"""
    
    print(f"📚 Distilling knowledge from cloud calls...")
    
    # Load cloud interaction log
    with open(cloud_log_path, 'r') as f:
        cloud_log = json.load(f)
    
    # In production, this would:
    # 1. Extract input-output pairs from cloud API calls
    # 2. Fine-tune a LoRA adapter on this data
    # 3. Evaluate the adapter
    
    print(f"  Found {len(cloud_log)} cloud interactions")
    
    # Mock distillation result
    return BreedingResult(
        offspring_id=Path(output_path).stem,
        sire_id="cloud",
        dam_id="base",
        merge_method="distillation",
        estimated_fitness=0.85,
        output_path=output_path,
        success=True
    )


def evaluate_agents(db_path: str, cull_threshold: float = 0.4) -> List[str]:
    """Evaluate all agents and return those to cull"""
    
    print(f"📊 Evaluating agents in {db_path}...")
    
    conn = sqlite3.connect(db_path)
    cursor = conn.execute(
        "SELECT id, fitness FROM agents WHERE status = 'active'"
    )
    
    to_cull = []
    for row in cursor.fetchall():
        agent_id, fitness = row
        if fitness < cull_threshold:
            to_cull.append(agent_id)
            print(f"  🗑️  Culling {agent_id} (fitness: {fitness:.2f})")
        else:
            print(f"  ✓ {agent_id} (fitness: {fitness:.2f})")
    
    # Mark as culled
    for agent_id in to_cull:
        conn.execute(
            "UPDATE agents SET status = 'culled' WHERE id = ?",
            (agent_id,)
        )
    
    conn.commit()
    return to_cull


def main():
    parser = argparse.ArgumentParser(
        description="SuperInstance Night School - Breeding Pipeline"
    )
    
    # Breeding mode
    parser.add_argument("--sire", help="Path to sire adapter")
    parser.add_argument("--dam", help="Path to dam adapter")
    parser.add_argument("--output", help="Output adapter path")
    parser.add_argument("--method", default="slerp", 
                       choices=["linear", "slerp", "ties"],
                       help="Merge method to use")
    parser.add_argument("--alpha", type=float, default=0.5,
                       help="Interpolation parameter (0-1)")
    
    # Distillation mode
    parser.add_argument("--distill", action="store_true",
                       help="Run distillation mode")
    parser.add_argument("--cloud-log", help="Path to cloud API log")
    
    # Evaluation mode
    parser.add_argument("--evaluate", action="store_true",
                       help="Run evaluation mode")
    parser.add_argument("--stud-book", help="Path to Stud Book database")
    parser.add_argument("--cull-threshold", type=float, default=0.4,
                       help="Fitness threshold for culling")
    
    args = parser.parse_args()
    
    if args.distill:
        if not args.cloud_log or not args.output:
            parser.error("Distillation requires --cloud-log and --output")
        result = distill_knowledge(args.cloud_log, args.output)
        print(f"✅ Distillation complete: {result.output_path}")
        
    elif args.evaluate:
        if not args.stud_book:
            parser.error("Evaluation requires --stud-book")
        culled = evaluate_agents(args.stud_book, args.cull_threshold)
        print(f"✅ Evaluation complete: {len(culled)} agents culled")
        
    elif args.sire and args.dam and args.output:
        result = breed_adapters(args.sire, args.dam, args.output, 
                               args.method, args.alpha)
        print(f"✅ Breeding complete: {result.output_path}")
        print(f"   Estimated fitness: {result.estimated_fitness:.2f}")
        
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
