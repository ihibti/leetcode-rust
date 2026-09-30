# Rust-Analyzer Setup Design

**Date:** 2026-02-07
**Status:** Approved

## Overview

Setting up rust-analyzer in Neovim with standard IDE features including code completion, inline type hints, documentation on demand, and code formatting.

## Requirements

- Automatic code completion via blink.cmp
- Inline type hints always visible
- `Cmd+Shift+I` to format code
- `Cmd+D` to show documentation and examples at cursor position
- Diagnostics (errors/warnings) displayed inline

## Approach

Since rust-analyzer is already installed via Cargo (`~/.cargo/bin/rust-analyzer`) and the Neovim config uses modern LSP setup patterns, we'll:

1. Enable rust-analyzer in the `servers` table with inlay hint configuration
2. Add to Mason installation list for management
3. Configure custom keybindings for formatting and documentation
4. Set inlay hints to show type hints, parameter names, and chaining hints

## Configuration Structure

### rust-analyzer Server Configuration

Add to `servers` table (~line 595 in init.lua):

```lua
local servers = {
  rust_analyzer = {
    settings = {
      ['rust-analyzer'] = {
        inlayHints = {
          bindingModeHints = { enable = false },
          chainingHints = { enable = true },
          closingBraceHints = { minLines = 10 },
          closureReturnTypeHints = { enable = "with_block" },
          parameterHints = { enable = true },
          typeHints = { enable = true },
        },
      },
    },
  },
}
```

### Mason Installation

Add `'rust-analyzer'` to the `ensure_installed` array (~line 617) for Mason to manage installation.

## Keybindings

Add inside the `LspAttach` autocmd callback (~line 580):

```lua
-- Format code with rust-analyzer
map('<D-S-i>', function() vim.lsp.buf.format() end, 'Format Document')

-- Show documentation and examples
map('<D-d>', vim.lsp.buf.hover, 'Show Documentation')
```

- `<D-S-i>` = `Cmd+Shift+I` → Format code via rust-analyzer
- `<D-d>` = `Cmd+D` → Show documentation popup

## Inlay Hints Behavior

Enabled features:
- **Type hints** - Shows inferred types for variables
- **Parameter hints** - Shows closure parameter types
- **Chaining hints** - Shows intermediate types in method chains

Disabled for clarity:
- Binding mode hints
- Closure return hints (except for block closures)

## Additional Features

- **Diagnostics** - Inline error/warning display
- **Quick fixes** - Suggested fixes when cursor is on error
- **Auto-imports** - Offers to add missing imports

## Testing Plan

1. Open a Rust file: `nvim src/main.rs`
2. Verify rust-analyzer initializes (check status line)
3. Test inlay hints appear automatically
4. Test `Cmd+D` shows documentation popup
5. Test `Cmd+Shift+I` formats code
6. Test completion suggestions appear while typing

## Troubleshooting

- Run `:Mason` to verify rust-analyzer installation
- Run `:LspInfo` to check rust-analyzer is attached
- Check `:messages` for error logs
