openssl genpkey -algorithm RSA -out jwt.key -pkeyopt rsa_keygen_bits:2048

# 2. Extract the corresponding public key
openssl rsa -in jwt.key -pubout -out jwt.pub