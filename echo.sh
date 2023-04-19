ENV_VARS="$(cat .env | awk '!/^\s*#/' | awk '!/^\s*$/')"

eval "$(
  printf '%s\n' "$ENV_VARS" | while IFS='' read -r line; do
    key=$(printf '%s\n' "$line"| sed 's/"/\\"/g' | cut -d '=' -f 1)
    value=$(printf '%s\n' "$line" | cut -d '=' -f 2- | sed 's/"/\\\"/g')
    printf '%s\n' "export $key=\"$value\""
  done
)"

flag=""
if [ "$RELEASE" = true ]
then
  flag="--release"
fi

cargo build $(flag) --bin echo
$MAELSTORM_DIR/maelstrom test -w echo --bin target/debug/echo --node-count 1 --time-limit 10