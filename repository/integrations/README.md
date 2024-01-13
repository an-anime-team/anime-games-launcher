# Anime Games Launcher integrations guide

Hello comrade developers! The new universal launcher uses lua scripts to handle games support. Here you can find a guide to write your own integration script, and a formal specification.

- [v1 standard specification](V1_SPECIFICATION.md)
- [v1 standard guide](V1_GUIDE.md)

## Manifest file

```json
{
	"manifest_version": "1",
	"game": {
		"name": "game-code-name",
		"title": "Formal Game Name",
		"developer": "Game Developer"
	},
	"script": {
		"path": "integration.lua",
		"version": "0.0.0",
		"standard": "1"
	}
}
```
