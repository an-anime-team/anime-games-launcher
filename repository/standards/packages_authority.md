# v1 format of the packages authority

Packages authority is a trusted source of metadata about the packages.
Authority provides list of packages hashes and their status.
Some packages could be marked as broken, insecure or malicious,
while others could be allowed to escape the luau engine sandbox
to perform specific files manipulations. Thus, authority indexes allow
launcher to dynamically load info about compromised or trusted modules.

It's recommended to specify hashes of actual luau modules, but you can also
specify hash of the whole package which will grant permissions to all the
inputs and outputs of this package. This both creates extra security concerns
and forces you to update authority index more frequently because hash will
change every time any of package resources is updated.

> `Localizable` type is defined in the games manifests standard.

```ts
type Manifest = {
    format: 1,

    // Title of the index.
    title: Localizable,

    // List of index resources.
    resources: ResourceInfo[]
};

type ResourceInfo = {
    // Title of the resource.
    // Example: "hpatchz"
    title: Localizable,

    // Short description of the resource (what it is used for).
    // Example: "Used to apply binary patches to game files."
    description?: Localizable,

    // Variants of this resource.
    // The same resource can update over time and change its hash.
    // Here you specify all the hashes of this resource and their status.
    variants: [hash: string]: ResourceStatus
};

type ResourceStatus = ResourceTrusted | ResourceCompromised | ResourceMalicious;

// Trusted resources are made by known people, proven to not contain
// any malicious code.
type ResourceTrusted = 'trusted' | {
    status: 'trusted',

    // List of APIs which are allowed to be used by the resource.
    privileges?: {
        // Process API allows luau module to run any binaries
        // or shell commands on the host system without sandboxing.
        process_api?: boolean
    },

    // List of paths which are allowed to be accessed by the luau module
    // in addition to default ones.
    allowed_paths?: string[]
};

// Compromised resources are general resources which were
// designed with good intentions but were proven to contain
// code exploitable by malicious actors. For example,
// a compromised resource can be a luau module with extended
// privileges which was using them for good purposes but
// contained a bug which could be abused by other luau modules
// without extended privileges to escape the sandbox themselves.
// Compromised resources don't have any special treatment.
// This category exists for statistical and UI purposes.
type ResourceCompromised = 'compromised' | {
    status: 'compromised',

    // URL to the page with detailed explanation.
    details_url: string
};

// Malicious resources are resources which were intentionally made
// to perform bad behavior on user system. These could be viruses
// or luau modules with hidden behavior.
type ResourceMalicious = 'malicious' | {
    status: 'malicious',

    // URL to the page with detailed explanation.
    details_url: string
};
```
