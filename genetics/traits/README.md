# 🧬 Gene Pool: Dynamic LoRA Traits

```
   ╭──────────────────────────────────────────────────────────────╮
   │                    GENE POOL ARCHITECTURE                     │
   │                                                              │
   │   Each trait is a hot-swappable LoRA adapter:                │
   │                                                              │
   │   genetics/traits/polite_tone/adapter.safetensors            │
   │   genetics/traits/json_output/adapter.safetensors            │
   │   genetics/traits/creative_writer/adapter.safetensors        │
   │                                                              │
   │   Load time: <200ms per trait                               │
   │   Swap time: <50ms                                           │
   │   Core binary: UNCHANGED (4.2 MB forever)                    │
   │                                                              │
   ╰──────────────────────────────────────────────────────────────╯
```

## How Dynamic LoRA Loading Works

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    DYNAMIC TRAIT LOADING                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   1. breed.md specifies traits:                                         │
│      ┌─────────────────────────────────────────────────────────────┐    │
│      │ | Gene Trait | Weight |                                    │    │
│      │ | polite_tone | 0.8   |                                    │    │
│      │ | json_output | 0.5   |                                    │    │
│      └─────────────────────────────────────────────────────────────┘    │
│                              │                                           │
│                              ▼                                           │
│   2. Collie reads trait weights:                                        │
│      ┌─────────────────────────────────────────────────────────────┐    │
│      │ trait_weights = {                                           │    │
│      │   "polite_tone": 0.8,                                       │    │
│      │   "json_output": 0.5                                        │    │
│      │ }                                                           │    │
│      └─────────────────────────────────────────────────────────────┘    │
│                              │                                           │
│                              ▼                                           │
│   3. LoRA composer merges adapters (<50ms):                            │
│      ┌─────────────────────────────────────────────────────────────┐    │
│      │ merged = compose(                                           │    │
│      │   load("polite_tone") * 0.8 +                              │    │
│      │   load("json_output") * 0.5                                 │    │
│      │ )                                                           │    │
│      └─────────────────────────────────────────────────────────────┘    │
│                              │                                           │
│                              ▼                                           │
│   4. Hot-swap into TensorRT engine:                                     │
│      ┌─────────────────────────────────────────────────────────────┐    │
│      │ engine.swap_adapter(merged)  # <50ms                        │    │
│      └─────────────────────────────────────────────────────────────┘    │
│                                                                          │
│   Result: Agent with new abilities, core binary unchanged.             │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Adding a New Trait

### Option 1: Use Pre-trained Trait

```bash
# Download a pre-trained LoRA adapter
mkdir -p genetics/traits/my_new_trait
cp ~/downloads/my_adapter.safetensors genetics/traits/my_new_trait/adapter.safetensors

# Create metadata
cat > genetics/traits/my_new_trait/meta.json << EOF
{
  "id": "my_new_trait",
  "name": "My New Trait",
  "description": "What this trait does",
  "size_bytes": 50000000,
  "compatible_species": ["Cattle"],
  "tags": ["style", "output"]
}
EOF

# Use in breed.md
# | my_new_trait | 0.7 | Description |
```

### Option 2: Train Custom Trait

```bash
# Train on your data
python night_school/breed.py \
  --base-model pasture/base_models/phi-3-mini \
  --data my_training_data.jsonl \
  --output genetics/traits/my_custom_trait

# Result: adapter.safetensors in genetics/traits/my_custom_trait/
```

## Available Traits

| Trait | Size | Description |
|:------|:-----|:------------|
| `polite_tone` | 50 MB | Professional, courteous communication |
| `casual_tone` | 45 MB | Relaxed, friendly communication |
| `json_output` | 30 MB | Structured JSON response format |
| `yaml_output` | 28 MB | YAML response format |
| `markdown_out` | 35 MB | Markdown-formatted output |
| `concise` | 40 MB | Brief, to-the-point responses |
| `verbose` | 42 MB | Detailed, explanatory responses |
| `creative_writer` | 55 MB | Creative, imaginative writing |
| `rust_coder` | 60 MB | Expert Rust code generation |
| `python_coder` | 58 MB | Expert Python code generation |

## Composing Multiple Traits

Traits can be composed with different weights:

```markdown
| Gene Trait | Weight | Description |
| :--- | :--- | :--- |
| `polite_tone` | `0.9` | Very formal |
| `concise` | `0.7` | Moderately brief |
| `json_output` | `0.3` | Light structure |
```

The composer merges these using the SLERP algorithm for smooth interpolation.

---

**Remember**: Adding traits NEVER changes the core binary size. All abilities are dynamic.
