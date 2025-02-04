# mochi

![GitHub last commit (by committer)](https://img.shields.io/github/last-commit/bloom-perf/mochi?logo=github)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/bloom-perf/mochi/ci.yml?style=flat&branch=main)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/bloom-perf/mochi/release.yml?label=publish)
[![dependency status](https://deps.rs/repo/github/Bloom-Perf/mochi/status.svg?path=%2F)](https://deps.rs/repo/github/Bloom-Perf/mochi?path=%2F)

![GitHub release (with filter)](https://img.shields.io/github/v/release/bloom-perf/mochi?style=flat)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=flat)](https://opensource.org/licenses/Apache-2.0)

Simple &amp; fast mock server written in rust.

## Overview

Mochi provides a unified way to create mock HTTP endpoints for testing and development. Instead of writing ad hoc servers for each case, you define your endpoints in YAML files. The server automatically builds routes based on your configuration, supports dynamic templating for responses, and even enables proxying of requests when needed.

## Problem Statement

In many testing and development environments, you may need to simulate various API endpoints that return static or dynamic responses. Traditional ad hoc solutions are often hard to maintain and scale. Mochi addresses this problem by:

- **Centralizing configuration:** Use structured YAML files to define endpoints, responses, and proxy rules.
- **Simplifying setup:** Automatically generates the required HTTP routes, avoiding boilerplate code.
- **Enabling flexibility:** Support both static responses and request proxying, along with dynamic templating for custom responses.

## Features

- **Lightweight and fast:** Built in Rust with a low memory footprint.
- **YAML-based configuration:** Define endpoints, headers, responses, and more in easy-to-read YAML files.
- **Dual-mode routing:** Supports both static endpoints (predefined responses) and proxy endpoints (forwarding requests to external services).
- **Response templating:** Integrate Handlebars templates to craft dynamic responses based on request headers, query parameters, path parameters, and body content.
- **Kubernetes Ready:** Includes a Helm chart and Dockerfile for containerized deployments.
- **Observability:** Integrated with OpenTelemetry to capture metrics and observability data.

### TO DO

- tracing
- dynamic mocking
- advanced latency profiles
- templating with request body access (json, xml...)

---

## Principles of URIs Formation

Mochi organizes its configuration around “systems” and “APIs”. Each system represents a logical group of endpoints, and APIs can be defined directly at the system level or within subdirectories (often representing versions or groups).

When the configuration is loaded, Mochi builds two main routers:

1. **Static Router:** Serves endpoints with preconfigured, static (or templated) responses.
2. **Proxy Router:** Forwards requests to a target URL as specified in the configuration.

### Static Endpoints

- **URI Structure:**  
  - **Root API:** If a system defines its API directly (e.g., via a single `api.yml`), the endpoints will be available under:

    ```bash
    /static/{system_name}/{route}
    ```

  - **Sub-APIs:** If a system contains subdirectories (e.g., `v1/`, `v2/`), each folder name becomes an API prefix. Endpoints are then accessible at:

    ```bash
    /static/{system_name}/{api_folder}/{route}
    ```

**Example:**

A system named `system` with a single API: `system/api.yml`

```yaml
rules:
  - matches: POST /route
    response: !OkJson "{ \"content\": \"hello\" }"
```

The endpoint is exposed at:

```bash
/static/system/route
```

If the system uses API subdirectories:

```markdown
system/
    v1/
        api.yml
    v2/
        api.yml
```

Then the endpoints will be:

```bash
/static/system/v1/{route defined in v1/api.yml}
/static/system/v2/{route defined in v2/api.yml}
```

### Proxy Endpoints

- **URI Structure:** Proxy endpoints are made available under the `/proxy` prefix. The format is as follows:

```bash
/proxy/{system_name}/{api_name}/{remaining_path}
```

- **How it works:** When a proxy is configured (via a `proxy.yml` file), requests to the corresponding proxy endpoint are forwarded to the target URL specified in the configuration. The remaining path (and any query parameters) is appended to the proxy URL.

**Example:**

A system named system with an API in the folder mvp that includes proxy configuration:

```markdown
system/
    mvp/
        api.yml
        proxy.yml
```

The proxy configuration file `proxy.yml` might look like this:

```yaml
url: http://example.com/api/
```

A GET request to the following endpoint:

```bash
/proxy/system/mvp/resource?query=123
```

Will be forwarded to:

```bash
http://example.com/api/resource?query=123
```

---

## Getting started

### Prerequisites

- **Rust:** Required for local development.
- **Helm & kubectl:** For Kubernetes deployments.
- **Docker:** For containerized builds.

### Installation

**1. Clone the repository:**

```bash
git clone https://github.com/Bloom-Perf/mochi.git
cd mochi
```

**2. Local Development:** Build and run Mochi with:

```bash
cargo run -- --config-path ./config
```

**3. Docker Build:** To build a Docker image, run:

```bash
docker build -t mochi .
```

Run the Docker container:

```bash
docker run -p 3000:3000 mochi
```

**4. Kubernetes Deployment:**

- Customize `helm/values.yaml`and `helm/config` as needed.
- Install the Helm chart:

```bash
helm install mochi ./helm
```

---

## Configuration Exemples

### Concepts

Mochi introduces a few simple concepts to organize its configuration

- **System** Folder with the system name, containing apis and data or directly an api.
- **Api** Set of endpoints defining a system behaviour. A system can define multiple Apis, that must be prefixed (api v1, v2...)
- **Rule** Mostly an endpoint with a response. The response can be simply inlined in the api if it is short enough, or accessed from the `data` folder
- **Data** Folder at the system level where responses are defined and can be used to craft different apis.
- **Response** Yaml file that describes the response of an endpoint

### Simple example

One system called `system` with a single api containing one single route accessible on `mochi/static/system/route`

```markdown
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

```markdown
system/
    v1/
        api.yml
    v2/
        api.yml
```

`v1/api.yml`:

```yaml api.yml
rules:
  - matches: POST  /route
    response: !OkJson "{content: \"hello\"}"
```

`v2/api.yml`:

```yaml api.yml
rules:
  - matches: PATCH  /another
    response: !OkJson "{content: \"hello\"}"
```

### Multiple apis with data folder example

One system called `system` with two apis located in subfolders accessible on `mochi/static/system/v1/route` and `mochi/static/system/v2/another`

```markdown
system/
    data/
        response.yml
    v1/
        api.yml
    v2/
        api.yml
```

`data/response.yml`:

```yaml response
status: 200
format: application/json
data: |
  {
    "static": "2"
  }
```

`v1/api.yml`:

```yaml api.yml
rules:
  - matches: POST  /route
    response: !File response
```

`v2/api.yml`:

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

`response.yml`:

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
