### 开发环境

- rustc: 1.67.0
- system: windows,linux,mac
- runtime: windows,linux,mac

### 运行

```
cargo run --bin bundle -- --create ...
```

### 编译

#### 安装docker

[docker文档](https://docs.docker.com/get-docker/)

#### 安装cross

[cross 文档](https://github.com/cross-rs/cross)

```bash
cargo install cross --git https://github.com/cross-rs/cross
```
#### 编译至目标平台

```bash
cross build --target x86_64-unknown-linux-musl --release
```

### openapi-codegen

- -c required open api config url(json)
- -o required output dir
- -s split module file
- -i ignore response field option

```bash
open-api-codegen -c <json config url> -o <output dir> -s <split file> -i <ignore option>
```