cargo install --path ./simulator

resim reset

OP1=$(resim new-account)
export privkey1=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export account1=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

export token=$(resim new-token-mutable resource_sim1qzxcrac59cy2v9lpcpmf82qel3cjj25v3k5m09rxurgqehgxzu | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '!d')

resim mint 200 $token --proofs 1,resource_sim1qzxcrac59cy2v9lpcpmf82qel3cjj25v3k5m09rxurgqehgxzu --manifest tx.rtm
resim run tx.rtm