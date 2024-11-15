# watchman install script

INSTALL_LOC=$1




# init working dirs
mkdir keys
mkdir logs



# init RSA keys

openssl genpkey -algorithm RSA -out $INSTALL_LOC/keys/jwt_pri.pem
openssl rsa -pubout -in $INSTALL_LOC/keys/jwt_pri.pem -out $INSTALL_LOC/keys/jwt_pub.pem

