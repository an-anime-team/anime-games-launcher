# Torrent API

BitTorrent protocol allows people to distribute files and folders in 
decentralized manner, fast and efficiently. This API allows you to fetch
torrent files info, download and seed them to other people.

Since BitTorrent protocol can be forbidden in some jurisdictions this API can
be disabled by the user even if module has access to it.

Note that torrent API doesn't remember previously added torrents.

| Function         | Description                                   |
| ---------------- | --------------------------------------------- |
| `torrent.create` | Create new torrent file.                      |
| `torrent.add`    | Add torrent to downloading queue.             |
| `torrent.list`   | List all the added torrents.                  |
| `torrent.info`   | Get information about added torrent.          |
| `torrent.pause`  | Pause added torrent downloading and seeding.  |
| `torrent.resume` | Resume added torrent downloading and seeding. |
| `torrent.delete` | Delete added torrent.                         |

## `torrent.create(path: string, [options: CreateTorrentOptions]) -> Promise<TorrentFile>`

Create new torrent file from provided path. This function may take some time to
calculate pieces' hashes for all the files, so a background promise is returned.

```ts
type CreateTorrentOptions = {
    name?: string;
    piece_size?: number;
    trackers?: string[];
};

type TorrentFile = {
    // Info hash of the torrent.
    info_hash: string;

    // Torrent magnet link.
    magnet: string;

    // Content of the torrent file.
    content: number[];
};
```

```luau
fs.write_file("test.txt", "Hello, World!")

local torrent_file = torrent.create("test.txt"):await()

print(`Info hash: {torrent_file.info_hash}`)
print(`Magnet link: {torrent_file.magnet}`)
```

## `torrent.add(torrent: string, [options: AddTorrentOptions]) -> Promise<string>`

Add torrent file, magnet link or info hash to the downloading queue and return 
added torrent's info hash string.

While torrent files contain all the necessary information about the torrent the
same can't be said for info hashes and magnet links. That's why a background
promise is returned that will resolve the info hash string only once all the
necessary metadata is downloaded from the network. For instant torrent additions
you can use torrent files.

```ts
type AddTorrentOptions = {
    // Path to a folder where the torrent should be downloaded. If unset, the
    // temporary folder is used.
    output_folder?: string;

    // List of extra trackers for this torrent.
    trackers?: string[];

    // Whether the torrent downloading should be started immediately.
    // Default: `false`.
    paused?: boolean;

    // Whether to restart the torrent if it's already added.
    // Default: `true`.
    restart?: boolean;
};
```

```luau
-- Magnet link to archlinux iso file
local magnet_link = "magnet:?xt=urn:btih:cdf37bb22c748fa8cb1594bdc39efed1bcd5cc31&dn=archlinux-2025.12.01-x86_64.iso"

-- The iso file will be downloaded to the `path.temp_dir()` folder since no
-- output folder is specified
torrent.add(magnet_link):await()
```

## `torrent.list() -> Promise<TorrentListInfo[]>`

List all the added torrents and some of their info, including info hashes.

```ts
type TorrentStats = {
    // Amount of downloaded (available) bytes.
    current: number;

    // Total amount of bytes.
    total: number;

    // Total amount of uploaded bytes.
    uploaded: number;
};

type TorrentListInfo = {
    // Name of the torrent. Some torrents may not have it.
    name?: string;

    // Info hash of the torrent.
    info_hash: string;

    // Torrent stats.
    stats: TorrentStats;

    // Whether the torrent downloading or seeding is paused.
    paused: boolean;

    // Whether the torrent's downloading is finished.
    finished: boolean;
};
```

```luau
for _, info in torrent.list():await() do
    print(`Hash: {info.info_hash}, name: {info.name}`)
end
```

## `torrent.info(info_hash: string) -> Promise<TorrentInfo | nil>`

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
    stats: TorrentStats;

    // Whether the torrent downloading or seeding is paused.
    paused: boolean;

    // Whether the torrent's downloading is finished.
    finished: boolean;
};
```

```luau
-- Read torrent info for previously added archlinux iso
local info = torrent.info("cdf37bb22c748fa8cb1594bdc39efed1bcd5cc31"):await()

dbg(info)
```

## `torrent.pause(info_hash: string) -> Promise<void>`

Pause added torrent downloading and seeding. Has no effect on torrents which
weren't added to downloading queue.

```luau
-- Pause archlinux iso downloading and seeding
torrent.pause("cdf37bb22c748fa8cb1594bdc39efed1bcd5cc31"):await()
```

## `torrent.resume(info_hash: string) -> Promise<void>`

Resume added torrent downloading and seeding. Has no effect on torrents which
weren't added to downloading queue.

```luau
-- Resume archlinux iso downloading and seeding
torrent.resume("cdf37bb22c748fa8cb1594bdc39efed1bcd5cc31"):await()
```
