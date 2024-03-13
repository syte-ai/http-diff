## http-diff

CLI tool to verify consistency across web server versions. Ideal for large-scale refactors, sanity tests and maintaining data integrity across versions.

Archives of precompiled binaries for http-diff are available for macOS and Linux on [every release](https://github.com/syte-ai/http-diff/releases).

[![Tests](https://github.com/syte-ai/http-diff/workflows/tests/badge.svg)](https://github.com/syte-ai/http-diff)
[![Crates.io](https://img.shields.io/crates/v/http-diff.svg)](https://crates.io/crates/http-diff)

Dual-licensed under MIT or the [UNLICENSE](https://unlicense.org/).

![UI demo](./assets/demo.gif)

The tool works by looking at the configuration file that can be specified by `--config` argument.

`http-diff --config=./config.json`

- `./config.json` - is the default value for this argument so it can be omitted.

## Config example:

```
{
  "domains": [
    {
      "domain": "http://domain-a.com/",
      "headers": {
        "cookie": "auth=secret"
      }
    },
    "http://stage.domain-a.com"
  ],
  "endpoints": [
    {
      "endpoint": "/health"
    },
    {
      "endpoint": "/api/v1/users/<userId>",
      "http_method": "GET",
      "response_processor": [
        "jq",
        "del(.body.timestamp)"
      ],
      "headers": {
        "X-custom": "custom-header"
      }
    },
    {
      "endpoint": "/api/v1/products?omit_id=<productId>&include_empty=<include_empty>",
      "request_builder": [
        "python3",
        "add_auth_to_requests.py"
      ],
      "variables": {
        "include_empty": [
          "true",
          "false"
        ]
      }
    }
  ],
  "variables": {
    "userId": [
      123,
      444
    ],
    "productId": "UUID"
  }
}
```

this config will be translated to following:

- `GET` request with `{"cookie":"auth=secret"}` in headers will be issued to `http://domain-a.com/health` and response be compared to response of `GET http://stage.domain-a.com/health` without any headers.

- Next endpoint `/api/v1/users/<userId>` has variable defined in it - `<userId>`. Anything within the brackets considered a variable name. In this case - `userId`.
  Variable then is looked up in the global variables property. In this case `userId` has two values: `123` and `444`. This will be mapped to following requests:

  - `GET http://domain-a.com/users/123` with `{"cookie":"auth=secret","X-custom":"custom-header"}` in headers and compared with `GET http://stage.domain-a.com/users/123` with `{"X-custom":"custom-header"}` in headers.
  - `GET http://domain-a.com/users/444` with `{"cookie":"auth=secret","X-custom":"custom-header"}` in headers and compared with `GET http://stage.domain-a.com/users/444` with `{"X-custom":"custom-header"}` in headers.

  Before comparing any response, `response_processor` will be applied. This endpoint has following preprocessor: `["jq", "del(.body.timestamp)"]`. Preprocessor is an external command that will be called with any arguments, before comparing responses. In this case after request to `http://domain-a.com/users/123` and request to `http://stage.domain-a.com/users/123` each response will passed to `jq 'del(.body.timestamp)'` which basically will remove the `timestamp` field from the body. `timestamp` field will be omitted from the compare process. `response_processor` can be any program, script or cli tool you can think of. Anything program that accepts `stdin`, and outputs to `stdout`. Output then will be compared. Not the original response.

- Next endpoint (`/api/v1/products?omit_id=&include_empty=<include_empty>`) will be mapped to:

  - `GET http://domain-a.com/api/v1/products?omit_id=22fae888-7bf9-45d6-87f4-83087cc80ba1&include_empty=true` with `{"cookie":"auth=secret"}` in headers and compared with `GET http://domain-a.com/api/v1/products?omit_id=22fae888-7bf9-45d6-87f4-83087cc80ba1&include_empty=true` without custom headers. `productId` variable has `UUID` value. This is a 'generator' variable. Random uuid will be generated and passed to both requests. `include_empty` variable has two values: `true` and `false`. These requests will use `true`, next one will be `false`.
  - `GET http://domain-a.com/api/v1/products?omit_id=4558903b-d4a6-46e8-869e-1b77677832f4&include_empty=false` with `{"cookie":"auth=secret"}` in headers and compared with `GET http://domain-a.com/api/v1/products?omit_id=4558903b-d4a6-46e8-869e-1b77677832f4&include_empty=false` without custom headers. `productId` gets new random uuid for both requests. `include_empty` becomes `false`.

  Before executing any requests, `request_builder` command will be applied on each request.
  In this case `python3 add_auth_to_requests.py` will be executed 4 times, before each request.
  `request_builder` gives you option to modify request in any possible way. We can add headers here, remove, modify the body, handle complicated authentication methods etc.
  Same as the `response_processor`, `request_builder` gets an stdin in the following structure, and expects stdout with the same structure:

  ```
  {
    "uri": "string",
    "http_method": "GET | POST | PUT | PATCH | DELETE",
    "headers": "key value headers map",
    "body": "string"
  }
  ```

## Requirements

- Latest rust installed when building from source

## Installation

Archives are available on [every release](https://github.com/syte-ai/http-diff/releases) as well as `.deb` files for Linux.
Autocomplete for arguments and man pages are included.

## Developing

- `cargo run` - for development
- `cargo test` - to run tests
- `cargo build -r` - to build in release mode
