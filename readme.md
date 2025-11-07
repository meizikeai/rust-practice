# rust-practice

## 项目构架

基于[axum](https://docs.rs/axum)、[sqlx](https://docs.rs/sqlx)、[redis](https://docs.rs/redis)构建，依赖[rust](https://www.rust-lang.org)环境。

## 项目地址

https://github.com/meizikeai/rust-practice.git

## 项目结构

| Path        | Description   | Notes   |
| ----------- | ------------- | ------- |
| src         | project       | --      |
| Cargo.toml  | package       | --      |

## 开发环境

  + 克隆项目 - `$ git clone https://github.com/meizikeai/rust-practice.git`
  + 启动项目 - `$ cd rust-practicet && cargo run`

## 项目说明

配置可见 src/utils/config.rs 文件。

```sh
# 测试示例
$ curl http://0.0.0.0:8887/get/888666/something
{"code":200,"data":{"fuck":1},"message":"OK"}

$ curl --location 'http://0.0.0.0:8887/del/888666/something' \
--header 'Content-Type: application/json' \
--data '{
    "fuck": 1,
}'
{"code":200,"data":{},"message":"OK"}
```

## 其它信息

Web Frameworks：https://www.arewewebyet.org/topics/frameworks
