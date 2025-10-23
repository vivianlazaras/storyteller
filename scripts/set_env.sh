
decrypt_keys() {
    local enc=${1:-./keys.tar.gz.enc}
    local dest=${2:-./keys}

    if [ ! -f "$enc" ]; then
        echo "Encrypted file not found: $enc" >&2
        return 1
    fi

    read -s -p "Enter decryption passphrase: " pass
    echo

    local tmp
    tmp=$(mktemp /tmp/keys.XXXXXX.tar.gz) || { echo "Failed to create temp file" >&2; return 2; }

    # feed passphrase on stdin to avoid exposing it in ps output
    if ! printf '%s' "$pass" | openssl enc -aes-256-cbc -d -pbkdf2 -iter 100000 -salt -in "$enc" -out "$tmp" -pass stdin 2>/dev/null; then
        echo "Decryption failed" >&2
        rm -f "$tmp"
        return 3
    fi

    mkdir -p "$dest"
    if ! tar -xzf "$tmp" -C "$dest"; then
        echo "Failed to extract archive" >&2
        rm -f "$tmp"
        return 4
    fi

    rm -f "$tmp"
    echo "Keys decrypted to $dest"
}

run_api_bg() {
    local api_dir=${1:-./api}
    local log=${2:-./api.log}
    local pidfile=${3:-./api.pid}

    if [ ! -d "$api_dir" ]; then
        echo "API directory not found: $api_dir" >&2
        return 1
    fi

    nohup bash -c "cd \"$api_dir\" && exec go run ." >> "$log" 2>&1 &
    echo $! > "$pidfile"

    sleep 0.5
    if kill -0 "$(cat "$pidfile")" 2>/dev/null; then
        echo "API started (pid $(cat "$pidfile")), logs: $log"
    else
        echo "Failed to start API; check $log" >&2
        return 2
    fi
}

encrypt_keys() {
    local src=${1:-./keys}
    local out=${2:-./keys.tar.gz.enc}

    if [ ! -e "$src" ]; then
        echo "Source not found: $src" >&2
        return 1
    fi

    read -s -p "Enter encryption passphrase: " pass1
    echo
    read -s -p "Confirm passphrase: " pass2
    echo

    if [ "$pass1" != "$pass2" ]; then
        echo "Passphrases do not match" >&2
        return 2
    fi

    local tmp
    tmp=$(mktemp /tmp/keys.XXXXXX.tar.gz) || { echo "Failed to create temp file" >&2; return 3; }

    if [ -d "$src" ]; then
        if ! tar -czf "$tmp" -C "$src" .; then
            echo "Failed to create archive" >&2
            rm -f "$tmp"
            return 4
        fi
    else
        if ! tar -czf "$tmp" -C "$(dirname "$src")" "$(basename "$src")"; then
            echo "Failed to create archive" >&2
            rm -f "$tmp"
            return 4
        fi
    fi

    # feed passphrase on stdin to avoid exposing it in ps output
    if ! printf '%s' "$pass1" | openssl enc -aes-256-cbc -e -pbkdf2 -iter 100000 -salt -in "$tmp" -out "$out" -pass stdin 2>/dev/null; then
        echo "Encryption failed" >&2
        rm -f "$tmp" "$out"
        return 5
    fi

    chmod 600 "$out" 2>/dev/null || true
    rm -f "$tmp"
    echo "Keys encrypted to $out"
}

export ISSUER_URL="http://auth.qrespite.org/realms/qrespite.org"
export CLIENT_ID=storyteller
export API_ENDPOINT="http://localhost:8080"
export CLIENT_SECRET=./secret
export REDIRECT_URL="http://qrespite.org:8000/"
../target/debug/storyteller "$@"
