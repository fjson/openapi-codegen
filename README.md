## OpenAPI 3 Support

### Supported Features

Currently supports only Swagger 3 (OpenAPI 3). Supported HTTP methods: `GET`, `POST`, `DELETE`, `PUT`.

### Development Environment

- **Rustc:** 1.67.0
- **Supported Systems:** Windows, Linux, macOS
- **Runtime:** Windows, Linux, macOS

### openapi-codegen CLI Options

- `-c` **(required)**: OpenAPI config URL (JSON)
- `-o` **(required)**: Output directory
- `-s` *(optional)*: Split module files
- `-i` *(optional)*: Ignore response field option
- `--tags` *(optional)*: Specify tags (comma-separated)
- `--wrap` *(optional)*: Add a wrapper type to response, e.g., `Wrap<Response>`
- `--namespace` *(optional)*: Add a namespace

```bash
open-api-codegen -c <json config url> -o <output dir> -s <split file> -i <ignore option> --tags <tag> --wrap <type name> --namespace <namespace>
```