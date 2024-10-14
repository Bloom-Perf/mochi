# mochi
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/bloom-perf/mochi/ci.yml?style=flat&branch=main)
![GitHub release (with filter)](https://img.shields.io/github/v/release/bloom-perf/mochi?style=flat)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=flat)](https://opensource.org/licenses/Apache-2.0)

Simple &amp; fast mock server written in rust.

## Overview

Mochi streamlines the mock server creation process, eliminating the need for ad hoc, basic applications just to set up a simple HTTP endpoint. It offers features like HandleBars templating for dynamic responses, yet maintains a focus on simplicity and ease of use. With Mochi, developers can quickly build effective mocks with an emphasis on thorough observability, making the development and testing process more efficient.

## Features
 - Fast, with a low footprint
 - Helm/Kubernetes/Docker ready
 - Simple Yaml API
 - OpenTelemetry ready
 - Templating with HandleBars

### TO DO
 - tracing
 - dynamic mocking
 - advanced latency profiles
 - templating with request body access (json, xml...)

## Getting started
### Prerequisites
- Install helm kubectl
- Rust for local development


### Helm - For now
- Clone the repo https://github.com/bloom-perf/mochi.git
- Copy `./helm` and adapt `./helm/values.yaml`
- Modify `./helm/config`
- `helm install`


## Configuration
### Concepts
Mochi introduces a few simple concepts to organize its configuration
- **System** Folder with the system name, containing apis and data or directly an api.
- **Api** Set of endpoints defining a system behaviour. A system can define multiple Apis, that must be prefixed (api v1, v2...)
- **Rule** Mostly an endpoint with a response. The response can be simply inlined in the api if it is short enough, or accessed from the `data` folder
- **Data** Folder at the system level where responses are defined and can be used to craft different apis.
- **Response** Yaml file that describes the response of an endpoint

### Simple example

One system called `system` with a single api containing one single route accessible on `mochi/static/system/route`

```
system/
    api.yml
```
**api.yml**
```yaml api.yml
rules:
  - matches: POST  /route
    response: !OkJson "{content: \"hello\"}"
```

### Multiple apis example

One system called `system` with two apis located in subfolders accessible on `mochi/static/system/v1/route` and `mochi/static/system/v2/another`

```
system/
    v1/
        api.yml
    v2/
        api.yml
```
**v1/api.yml**
```yaml api.yml
rules:
  - matches: POST  /route
    response: !OkJson "{content: \"hello\"}"
```
**v2/api.yml**
```yaml api.yml
rules:
  - matches: PATCH  /another
    response: !OkJson "{content: \"hello\"}"
```

### Multiple apis with data folder example

One system called `system` with two apis located in subfolders accessible on `mochi/static/system/v1/route` and `mochi/static/system/v2/another`

```
system/
    data/
        response.yml
    v1/
        api.yml
    v2/
        api.yml
```
**data/response.yml**
```yaml response
status: 200
format: application/json
data: |
  {
    "static": "2"
  }
```
**v1/api.yml**
```yaml api.yml
rules:
  - matches: POST  /route
    response: !File response
```
**v2/api.yml**
```yaml api.yml
rules:
  - matches: PATCH  /another
    response: !File response
```

### Response body templating
You can build your response based on some request data, and the [Handlebars](http://handlebarsjs.com/) templating system.

Access
- request headers with the `headers.` prefix like this `{{headers.my_header}}`
- request url path parameters with `url.path.` prefix like this `{{url.path.my_path_parameter}}`
- request url query parameters with `url.query.` prefix like this `{{url.query.my_query_parameter}}`

**response.yml**
```yaml
status: 200
format: application/json
data: |
  {
    "static": "2",
    "headers.header": "{{headers.header}}",
    "url.query.foo": {{#if url.query.foo }}"{{url.query.foo}}"{{else}}"none"{{/if}},
    "url.path.path_param": "{{ url.path.path_param }}",
    "unknown parameter url.test.test": "{{url.test.test}}"
  }
```


### Other examples
See `./tests/*`

### Environment variables
- `CONFIG_PATH`: specify the path where the configuration of the mock server is located (specify `./helm/config` to work locally)
