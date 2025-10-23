# Integrations

Integrations can be thought of as plugins, but for the sake of added security, and reproducability to integration system leverages wasmtime to run web assembly system interface (WASI) complient code rather than running native plugins that could potentially crash the main program, or expose additional vulnerabilities.

## Advantages of this approach:
1. Security
It is harder for insecure code to effect the running state of the core program, allowing an integration to be easily offloaded if it fails. 
2. Privacy
The plugin system using wasmtime allows for whitelisting web services, and routes that the integration is allowed to use prohibiting calls to analytics API's and routes without impacting the core functioning of the integration.
3. Reproducability
An integration is more reproducable because targeting WASI requires either full static linking, or an assurance from this code base that the needed dependency is available.

## Building an Integration
