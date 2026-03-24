# 🐄 Breed: Template-Cow

> **Copy this folder to create a new ability in 60 seconds.**
> ```bash
> cp -r pasture/cattle/template pasture/cattle/my-new-ability
> ```

## 📋 Overview

[Describe what this agent does in 1-2 sentences]

## 🧬 Genetic Composition

| Gene Trait | Weight | Description |
| :--- | :--- | :--- |
| `trait_name` | `0.8` | [What this trait does] |
| `another_trait` | `0.5` | [Description] |

## 🧠 Genetic Code (System Prompt)

```
You are a [Role Description].

Your responsibilities:
1. [First responsibility]
2. [Second responsibility]
3. [Third responsibility]

Communication style:
- [Style guideline 1]
- [Style guideline 2]

Output format:
- [Expected output format]
```

## 🔧 Setup

```bash
# Copy this breed to your pasture
cp -r pasture/cattle/template pasture/cattle/[your-breed-name]

# Edit the breed.md
nano pasture/cattle/[your-breed-name]/breed.md

# Restart the Ranch (or hot-reload will pick it up)
make run
```

## 📊 Example Outputs

### Input
```
[Example input]
```

### Output
```
[Example output]
```

---

## Notes

- **Load time**: <200ms (hot-reload)
- **Core binary**: Unchanged (4.2 MB forever)
- **Extensibility**: This file only, no Rust edits needed
