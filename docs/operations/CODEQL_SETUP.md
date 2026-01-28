# CodeQL Setup Instructions

## Current Status

The repository uses **Advanced Setup** for CodeQL analysis via the workflow file `.github/workflows/ci.yml`.

## Warning Resolution

If you see the warning:
```
1 configuration not found
Warning: Code scanning cannot determine the alerts introduced by this pull request, because 1 configuration present on refs/heads/main was not found: Default setup
```

This occurs because the repository has both "Default setup" (configured in GitHub UI) and "Advanced setup" (workflow file) enabled.

## Solution: Disable Default Setup

To resolve this warning and use only Advanced Setup:

1. Navigate to your repository on GitHub
2. Go to **Settings** → **Code security and analysis** (or **Security** → **Code scanning**)
3. Find **CodeQL analysis** in the list
4. Click the menu (•••) next to "CodeQL analysis"
5. Select **"Switch to advanced"** or **"Disable CodeQL"**
6. Confirm the action

After disabling Default Setup, only the Advanced Setup workflow (`.github/workflows/ci.yml`) will run.

## Verification

After disabling Default Setup:
- The warning should disappear on future PRs
- CodeQL will run only via the workflow file
- You'll have full control over CodeQL configuration

## Current Configuration

The Advanced Setup workflow:
- Runs on every push and pull request
- Analyzes Rust code
- Uses security and quality queries
- Has proper permissions (`security-events: write`)

## No Action Required for PR Merge

**Important**: This warning does **not** block PR merges. The CodeQL analysis is working correctly. The warning is informational and indicates a configuration mismatch between branches.
