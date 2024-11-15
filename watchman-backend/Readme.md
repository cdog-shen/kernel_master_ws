# Dependence

## Rember diesel on Windows

1. download a mysql-community-server ZIP pac

    I choose the latest version [(9.1.0)](https://cdn.mysql.com/archives/mysql-9.0/mysql-9.0.1-winx64.zip)

    Or the C++ connector, I think it is OKEY but I'm too tired to try it `:(`

2. create a copy of **mysql-9.0.1-winx64\lib\mysqlclient.lib** and name it **mysql-9.0.1-winx64\lib\libmysqlclient.lib**

    Just into lib dir do this `cp mysqlclient.lib libmysqlclient.lib`

3. add 2 Environment variables

    `MYSQLCLIENT_LIB_DIR` = `C:\Program Files\MySQL\mysql-9.0.1-winx64\lib` (Change it to ur mysql path)

    `MYSQLCLIENT_VERSION` = `8.0.30` (I don't Know why they only support specifying a few versions !!! 8.0.30 it currently latest)

4. done. u can install diesel_cli with mysql feature

    `cargo install diesel_cli --no-default-features --features "mysql"`
