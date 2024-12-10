# watchman install script

# var set
INSTALL_LOC=$1
DATABASE_URL=$2




# init working dirs
mkdir keys
mkdir logs



# init RSA keys
openssl genpkey -algorithm RSA -out $INSTALL_LOC/keys/jwt_pri.pem
openssl rsa -pubout -in $INSTALL_LOC/keys/jwt_pri.pem -out $INSTALL_LOC/keys/jwt_pub.pem

# init DB tables
diesel migration generate --database-url $DATABASE_URL init_migration --diff-schema
diesel migration run --database-url $DATABASE_URL
