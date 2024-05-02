# RCLI

rcli is rust cli

## 作业一

* rcli text encrypt -key"xxx"> 加密并输出 base64

```shell
❯ cargo run -- text encrypt --key fixtures/chacha20poly1305.txt
    Finished dev [unoptimized + debuginfo] target(s) in 0.09s
     Running `target/debug/rcli text encrypt --key fixtures/chacha20poly1305.txt`
helloworld!^D
vUWe-RSVjrL7Sj3yFGUVXjOL8l8eTgGxR_dt

```

* rcli text decrypt -key"XXX" >base64 > binary> 解密文本

```shell
❯ cargo run -- text decrypt --key fixtures/chacha20poly1305.txt
    Finished dev [unoptimized + debuginfo] target(s) in 0.09s
     Running `target/debug/rcli text decrypt --key fixtures/chacha20poly1305.txt`
vUWe-RSVjrL7Sj3yFGUVXjOL8l8eTgGxR_dt^D
helloworld!

❯ cargo run -- text decrypt --key fixtures/chacha20poly1305.txt -i fixtures/chacha20poly1305-decrypt.txt
    Finished dev [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/rcli text decrypt --key fixtures/chacha20poly1305.txt -i fixtures/chacha20poly1305-decrypt.txt`

helloworld!


❯ cargo run -- text decrypt --key fixtures/chacha20poly1305.txt -i fixtures/chacha20poly1305-decrypt-b64.txt
    Finished dev [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/rcli text decrypt --key fixtures/chacha20poly1305.txt -i fixtures/chacha20poly1305-decrypt-b64.txt`

helloworld!

```

## 作业二

* sign

```shell
❯ cargo run -- jwt sign --sub acme --aud device --exp 14d --key fixtures/jwt-secret.txt
    Finished dev [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/rcli jwt sign --sub acme --aud device --exp 14d --key fixtures/jwt-secret.txt`
eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhY21lIiwiYXVkIjoiZGV2aWNlIiwiZXhwIjoxNzE1ODc1ODc5fQ.zOBNEPSonPqLcszMMkCw2UiuSWl5XJztTHDy4Bi9ihk
```

![img_1.png](img_1.png)

* verify

```shell
❯ cargo run -- jwt verify --key fixtures/jwt-secret.txt --token eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhY21lIiwiYXVkIjoiZGV2aWNlIiwiZXhwIjoxNzE1ODc1ODc5fQ.zOBNEPSonPqLcszMMkCw2UiuSWl5XJztTHDy4Bi9ihk --aud device
    Finished dev [unoptimized + debuginfo] target(s) in 0.07s
     Running `target/debug/rcli jwt verify --key fixtures/jwt-secret.txt --token eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhY21lIiwiYXVkIjoiZGV2aWNlIiwiZXhwIjoxNzE1ODc1ODc5fQ.zOBNEPSonPqLcszMMkCw2UiuSWl5XJztTHDy4Bi9ihk --aud device`
Verified: true
Claim { sub: "acme", aud: "device", exp: 1715875879 }
````

## 作业三

```shell
❯ cargo run -- http serve
   Compiling rcli v0.1.0 (/Users/cxn/rust/geektime-rust/rcli)
    Finished dev [unoptimized + debuginfo] target(s) in 0.64s
     Running `target/debug/rcli http serve`
```

```shell
curl 127.0.0.1:8080
<html><body><ul><li><a href="_typos.toml">_typos.toml</a></li><li><a href="Cargo.toml">Cargo.toml</a></li><li><a href="deny.toml">deny.toml</a></li><li><a href="CHANGELOG.md">CHANGELOG.md</a></li><li><a href=".pre-commit-config.yaml">.pre-commit-config.yaml</a></li><li><a href="target/">target/</a></li><li><a href="test.http">test.http</a></li><li><a href="Cargo.lock">Cargo.lock</a></li><li><a href="README.md">README.md</a></li><li><a href="cliff.toml">cliff.toml</a></li><li><a href=".gitignore">.gitignore</a></li><li><a href=".github/">.github/</a></li><li><a href="fixtures/">fixtures/</a></li><li><a href=".git/">.git/</a></li><li><a href="assets/">assets/</a></li><li><a href=".idea/">.idea/</a></li><li><a href="src/">src/</a></li></ul></body></html>%
```

![img.png](img.png)
