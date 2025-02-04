# 🔄 GitHub Workflows Documentation

This directory contains our GitHub Actions workflows for continuous integration and workflow management.

## 🍡 Mochi CI Workflow

The main CI pipeline for Rust projects (`ci.yml`).

### 📋 Pipeline Steps

1. 🔍 **Check**
   - Basic Rust compilation check
   - Always runs first

2. 📝 **Format**
   - Checks code formatting with `rustfmt`
   - Skip with `[skip-fmt]` in commit message

3. 🔬 **Clippy**
   - Runs Rust linter
   - Skip with `[skip-clippy]` in commit message

4. 🧪 **Tests**
   - Executes test suite
   - Skip with `[skip-tests]` in commit message

### 💡 Usage Examples

Skip specific checks in commit messages:

```bash
git commit -m "🐛 fix: Update documentation [skip-tests]"
git commit -m "📝 docs: Format README [skip-fmt] [skip-clippy]"
```

## 🎯 Paths Ignore Management

The CI workflow uses a repository variable to manage ignored paths.
To update ignored paths, modify the `IGNORE_PATTERNS` variable in repository settings, and run
the ``update-patterns`` pipeline.

### 📁 Structure

```plaintext
.github/
  └── workflows/
      └── update-patterns.yml
```

### ⚙️ Configuration

Repository Variable Setup:

1. Go to Repository Settings > Secrets and Variables > Actions > Variables
2. Create variable named `IGNORE_PATTERNS`
3. Add patterns, one per line:

   ```plaintext
   *.md
   docs/**
   .github/**
   LICENSE
   target/
   Cargo.lock
   ```

### 📝 Default Ignored Paths

Common patterns to consider:

- `*.md` - Markdown files
- `docs/**` - Documentation directory
- `.github/**` - GitHub configuration
- `LICENSE` - License file
- `target/` - Build outputs
- `Cargo.lock` - Dependencies lock file

## 🤝 Contributing

When modifying CI workflows:

1. Test changes in a branch
2. Update this documentation if needed
3. Create a PR with clear description

## 🚨 Troubleshooting

Common issues and solutions:

1. **Skip Keywords Not Working**
   - Ensure exact spelling: `[skip-fmt]`, `[skip-clippy]`, `[skip-tests]`
   - Place keywords in commit message, not PR title

2. **Path Ignore Patterns Not Working**
   - Check `IGNORE_PATTERNS` variable format in repository settings
   - Ensure each pattern is on a new line
   - Verify pattern syntax follows glob format

## 📊 Pipeline Summary

The CI pipeline automatically generates a summary including:

- List of skipped steps
- Job status overview
- Build and test results

Find the summary in the workflow run details under "Summary" section.
