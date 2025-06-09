read -s -p "Enter password to hash: " PASSWORD
echo

# Generate bcrypt hash using htpasswd (requires apache2-utils or httpd-tools)
HASH=$(htpasswd -nbB user "$PASSWORD" | cut -d: -f2)

echo "Bcrypt hash: $HASH"