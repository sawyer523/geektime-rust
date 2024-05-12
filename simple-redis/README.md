# Geektime Rust 语言训练营

## 为 simple-redis 实现你想实现的命令，比如：

* echo command:  https://redis.io/commands/echo/
* hmget command:  https://redis.io/commands/hmget/

server 端：

```shell
2024-05-12T15:44:47.118554Z  INFO simple_redis: Simple-Redis-Server is listening on 0.0.0.0:6379
2024-05-12T15:44:58.556138Z  INFO simple_redis: Accepted connection from 127.0.0.1:59914
2024-05-12T15:44:58.556405Z  INFO simple_redis::network: Received frame: Array(RespArray(Some([BulkString(BulkString(Some([67, 79, 77, 77, 65, 78, 68]))), BulkString(BulkString(Some([68, 79, 67, 83])))])))
2024-05-12T15:44:58.556675Z  INFO simple_redis::network: Executing command: Unrecognized(Unrecognized)
2024-05-12T15:44:58.556701Z  INFO simple_redis::network: Sending response: SimpleString(SimpleString("OK"))
2024-05-12T15:45:12.558530Z  INFO simple_redis::network: Received frame: Array(RespArray(Some([BulkString(BulkString(Some([101, 99, 104, 111]))), BulkString(BulkString(Some([104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100])))])))
2024-05-12T15:46:03.075443Z  INFO simple_redis: Accepted connection from 127.0.0.1:60077
2024-05-12T15:46:03.075583Z  INFO simple_redis::network: Received frame: Array(RespArray(Some([BulkString(BulkString(Some([67, 79, 77, 77, 65, 78, 68]))), BulkString(BulkString(Some([68, 79, 67, 83])))])))
2024-05-12T15:46:03.075628Z  INFO simple_redis::network: Executing command: Unrecognized(Unrecognized)
2024-05-12T15:46:03.075647Z  INFO simple_redis::network: Sending response: SimpleString(SimpleString("OK"))
2024-05-12T15:46:04.806581Z  INFO simple_redis::network: Received frame: Array(RespArray(Some([BulkString(BulkString(Some([101, 99, 104, 111]))), BulkString(BulkString(Some([104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100])))])))
2024-05-12T15:46:04.806635Z  INFO simple_redis::network: Executing command: Echo(Echo { value: "hello world" })
2024-05-12T15:46:04.806654Z  INFO simple_redis::network: Sending response: SimpleString(SimpleString("\"hello world\""))
2024-05-12T15:46:17.482251Z  INFO simple_redis::network: Received frame: Array(RespArray(Some([BulkString(BulkString(Some([104, 109, 115, 101, 116]))), BulkString(BulkString(Some([109, 121, 104, 97, 115, 104]))), BulkString(BulkString(Some([49]))), BulkString(BulkString(Some([50]))), BulkString(BulkString(Some([51]))), BulkString(BulkString(Some([52]))), BulkString(BulkString(Some([53]))), BulkString(BulkString(Some([54]))), BulkString(BulkString(Some([55]))), BulkString(BulkString(Some([56])))])))
2024-05-12T15:46:17.482347Z  INFO simple_redis::network: Executing command: HMSet(HMSet { key: "myhash", fields: RespArray(Some([BulkString(BulkString(Some([49]))), BulkString(BulkString(Some([50]))), BulkString(BulkString(Some([51]))), BulkString(BulkString(Some([52]))), BulkString(BulkString(Some([53]))), BulkString(BulkString(Some([54]))), BulkString(BulkString(Some([55]))), BulkString(BulkString(Some([56])))])) })
2024-05-12T15:46:17.482459Z  INFO simple_redis::network: Sending response: SimpleString(SimpleString("OK"))
2024-05-12T15:46:28.129893Z  INFO simple_redis::network: Received frame: Array(RespArray(Some([BulkString(BulkString(Some([104, 109, 103, 101, 116]))), BulkString(BulkString(Some([109, 121, 104, 97, 115, 104]))), BulkString(BulkString(Some([49]))), BulkString(BulkString(Some([50]))), BulkString(BulkString(Some([51]))), BulkString(BulkString(Some([52]))), BulkString(BulkString(Some([53]))), BulkString(BulkString(Some([54]))), BulkString(BulkString(Some([55]))), BulkString(BulkString(Some([56]))), BulkString(BulkString(Some([57]))), BulkString(BulkString(Some([49, 48])))])))
2024-05-12T15:46:30.539645Z  INFO simple_redis::network: Executing command: HMGet(HMGet { key: "myhash", fields: RespArray(Some([BulkString(BulkString(Some([49]))), BulkString(BulkString(Some([50]))), BulkString(BulkString(Some([51]))), BulkString(BulkString(Some([52]))), BulkString(BulkString(Some([53]))), BulkString(BulkString(Some([54]))), BulkString(BulkString(Some([55]))), BulkString(BulkString(Some([56]))), BulkString(BulkString(Some([57]))), BulkString(BulkString(Some([49, 48])))])) })
2024-05-12T15:46:30.539749Z  INFO simple_redis::network: Sending response: Array(RespArray(Some([BulkString(BulkString(Some([50]))), Null(RespNull), BulkString(BulkString(Some([52]))), Null(RespNull), BulkString(BulkString(Some([54]))), Null(RespNull), BulkString(BulkString(Some([56]))), Null(RespNull), Null(RespNull), Null(RespNull)])))
2024-05-12T15:46:38.138795Z  INFO simple_redis::network: Received frame: Array(RespArray(Some([BulkString(BulkString(Some([104, 109, 103, 101, 116]))), BulkString(BulkString(Some([109, 121, 104, 97, 115, 104]))), BulkString(BulkString(Some([49]))), BulkString(BulkString(Some([50]))), BulkString(BulkString(Some([51]))), BulkString(BulkString(Some([52]))), BulkString(BulkString(Some([53]))), BulkString(BulkString(Some([54]))), BulkString(BulkString(Some([55]))), BulkString(BulkString(Some([56]))), BulkString(BulkString(Some([57]))), BulkString(BulkString(Some([49, 48])))])))
2024-05-12T15:46:38.138906Z  INFO simple_redis::network: Executing command: HMGet(HMGet { key: "myhash", fields: RespArray(Some([BulkString(BulkString(Some([49]))), BulkString(BulkString(Some([50]))), BulkString(BulkString(Some([51]))), BulkString(BulkString(Some([52]))), BulkString(BulkString(Some([53]))), BulkString(BulkString(Some([54]))), BulkString(BulkString(Some([55]))), BulkString(BulkString(Some([56]))), BulkString(BulkString(Some([57]))), BulkString(BulkString(Some([49, 48])))])) })
2024-05-12T15:46:38.139027Z  INFO simple_redis::network: Sending response: Array(RespArray(Some([BulkString(BulkString(Some([50]))), Null(RespNull), BulkString(BulkString(Some([52]))), Null(RespNull), BulkString(BulkString(Some([54]))), Null(RespNull), BulkString(BulkString(Some([56]))), Null(RespNull), Null(RespNull), Null(RespNull)])))
```

client:

```shell
❯ redis-cli
127.0.0.1:6379> echo "hello world"
"hello world"
127.0.0.1:6379> hmset myhash 1 2 3 4 5 6 7 8
OK
127.0.0.1:6379> hmget myhash 1 2 3 4 5 6 7 8 9 10
 1) "2"
 2) (nil)
 3) "4"
 4) (nil)
 5) "6"
 6) (nil)
 7) "8"
 8) (nil)
 9) (nil)
10) (nil)
(2.41s)
127.0.0.1:6379> hmget myhash 1 2 3 4 5 6 7 8 9 10
 1) "2"
 2) (nil)
 3) "4"
 4) (nil)
 5) "6"
 6) (nil)
 7) "8"
 8) (nil)
 9) (nil)
10) (nil)
127.0.0.1:6379>
```

## 重构代码：

[删除 NullBulkString / NullArray](./src/resp/bulk_string.rs)

[重构 BulkString / RespArray 代码](./src/resp/array.rs)