export ISSUER_URL="http://auth.qrespite.org/realms/qrespite.org"
export CLIENT_ID=storyteller
export API_ENDPOINT="http://localhost:8080"
export CLIENT_SECRET=./secret
export REDIRECT_URL="http://qrespite.org:8000/"
./target/debug/storyteller "$@"
