# Torrent API

BitTorrent protocol allows people to distribute files and folders in 
decentralized manner, fast and efficiently. This API allows you to fetch
torrent files info, download and seed them to other people.

Since BitTorrent protocol can be forbidden in some jurisdictions this API can
be disabled by the user even if module has access to it.

| Function       | Description                       |
| -------------- | --------------------------------- |
| `torrent.add`  | Add torrent to downloading queue. |
| `torrent.get`  | Get torrent handle from its hash. |
| `torrent.info` | Get information about a torrent.  |

## `torrent.add(uri: string, [options: AddTorrentOptions]) -> number`

Add torrent to the downloading queue. A torrent URI can be either a path to a
torrent file or a magnet link.

```ts
type AddTorrentOptions = {
    // Path to a folder where the torrent should be downloaded. If unset, the
    // temporary folder is used.
    output_folder?: string;
};
```

```luau
-- Magnet link to archlinux iso file
local magnet_link = "magnet:?xt=urn:btih:cdf37bb22c748fa8cb1594bdc39efed1bcd5cc31&dn=archlinux-2025.12.01-x86_64.iso"

-- The iso file will be downloaded to the `path.temp_dir()` folder since no
-- output folder is specified
torrent.add(magnet_link)
```
