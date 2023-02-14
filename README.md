### 开发环境

- rustc: 1.67.0
- system: windows,linux,mac
- runtime: windows,linux,mac

### openapi-codegen

- -c required open api config url(json)
- -o required output dir
- -s split module file
- -i ignore response field option

```bash
open-api-codegen -c <json config url> -o <output dir> -s <split file> -i <ignore option>
```