# settings

Manage declarch settings.

## Usage

```bash
declarch settings <COMMAND>
```

## Commands

| Command | Description |
|---------|-------------|
| `set <key> <value>` | Set a value |
| `get <key>` | Get a value |
| `show` | Show all settings |
| `reset <key>` | Reset to default |

## Examples

### Show All Settings

```bash
declarch settings show
```

### Set Setting

```bash
declarch settings set color never
declarch settings set format json
```

### Get Setting

```bash
declarch settings get color
```

### Reset Setting

```bash
declarch settings reset color
```

## Available Settings

| Setting | Values | Description |
|---------|--------|-------------|
| `color` | `auto`, `always`, `never` | Terminal colors |
| `format` | `table`, `json`, `yaml` | Output format |
| `progress` | `auto`, `always`, `never` | Progress bars |
