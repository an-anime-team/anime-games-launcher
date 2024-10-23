# v1 format of the packages authority

Packages authority is a trusted source of metadata about the packages.
Authority provides list of packages hashes and their status.
Some packages could be marked as broken, insecure or malicious,
while others could be allowed to escape the luau engine sandbox
to perform specific files manipulations. Thus, authority indexes allow
launcher to dynamically load info about compromised or trusted modules.

> `Localizable` type is defined in the games manifests standard.

```ts
type Manifest = {
    format: 1,

    // Title of the index.
    title: Localizable,

    // Table of base32 hashes of resources and their info.
    resources: [hash: string]: ResourceInfo
};

// Compromised resources are general resources which were
// designed with good intentions but were proved to contain
// code exploitable by malicious actors. For example,
// a compromised resource can be a luau module with extended
// privileges which was using them for good purposes but
// contained a bug which could be abused by other luau modules
// without extended privileges to escape the sandbox themselves.
// Compromised resources don't have any special treatment.
// This category exists for statistical and UI purposes.
type ResourceCompromised = 'compromised' | {
    type: 'compromised',

    // URL to the page with detailed explanation.
    details_url: string
};

// Malicious resources are resources which were intentionally made
// to perform bad behavior on user system. These could be viruses
// or luau modules with hidden behavior.
type ResourceMalicious = 'malicious' | {
    type: 'malicious',

    // URL to the page with detailed explanation.
    details_url: string
};

// Trusted resources are made by known people, proved
// to not contain any malicious code and which require extended
// privileges to function.
type ResourceTrusted = {
    type: 'trusted',

    // List of APIs which are allowed to be used by the resource.
    extended_privileges: {
        // Process API allows luau module to run any binaries
        // or shell commands on the host system without sandboxing.
        process_api: boolean
    }
};

type ResourceInfo = {
    // Title of the resource.
    // Example: "hpatchz"
    title: Localizable,

    // Short description of the resource (what it is used for).
    // Example: "Used to apply binary patches to game files."
    description: Localizable,

    // Status of the resource.
    status: ResourceCompromised | ResourceMalicious | ResourceTrusted
};
```
