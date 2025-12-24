# Torrent API

BitTorrent protocol allows people to distribute files and folders in 
decentralized manner, fast and efficiently. This API allows you to fetch
torrent files info, download and seed them to other people.

Since BitTorrent protocol can be forbidden in some jurisdictions this API can
be disabled by the user even if module has access to it.

| Function       | Description                          |
| -------------- | ------------------------------------ |
| `torrent.add`  | Add torrent to downloading queue.    |
| `torrent.info` | Get information about added torrent. |

## `torrent.add(torrent: string, [options: AddTorrentOptions]) -> string`

Add torrent file, magnet link or info hash to the downloading queue and return 
added torrent's info hash string.

```ts
type AddTorrentOptions = {
    // Path to a folder where the torrent should be downloaded. If unset, the
    // temporary folder is used.
    output_folder?: string;

    // Whether the torrent downloading should be started immediately.
    paused?: boolean;
};
```

```luau
-- Magnet link to archlinux iso file
local magnet_link = "magnet:?xt=urn:btih:cdf37bb22c748fa8cb1594bdc39efed1bcd5cc31&dn=archlinux-2025.12.01-x86_64.iso"

-- The iso file will be downloaded to the `path.temp_dir()` folder since no
-- output folder is specified
torrent.add(magnet_link)
```

## `torrent.info(info_hash: string) -> TorrentInfo | nil`

Get information about already added torrent using its info hash. Return `nil`
if there's no torrent with provided info hash.

```ts
type TorrentPeerInfo = {
    // Address of the peer.
    address: string;

    // Amount of bytes downloaded from this peer.
    downloaded: number;
};

type TorrentFileInfo = {
    // Relative path of a file.
    path: string;

    // Size of the file.
    size: number;
};

type TorrentInfo = {
    // Name of the torrent. Some torrents may not have it.
    name?: string;

    // List of torrent trackers.
    trackers: string[];

    // List of active torrent peers.
    peers: TorrentPeerInfo[];

    // Files of the torrent.
    files: TorrentFileInfo[];

    // Torrent stats.
    stats: {
        // Amount of downloaded (available) bytes.
        current: number;

        // Total amount of bytes.
        total: number;

        // Total amount of uploaded bytes.
        uploaded: number;
    };

    // Whether the torrent downloading or seeding is paused.
    paused: boolean;

    // Whether the torrent's downloading is finished.
    finished: boolean;
};
```

```luau
-- Read torrent info for previously added archlinux iso
local info = torrent.info("cdf37bb22c748fa8cb1594bdc39efed1bcd5cc31")

dbg(info)
```
