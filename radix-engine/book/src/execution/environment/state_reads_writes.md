# State Reads/Writes

State Reads and Writes from objects may be only be done by the current actor using the following system calls:
* `actor_open_field`
* `actor_open_key_value_entry`
* `actor_index_insert`
* `actor_index_remove`
* `actor_index_scan_keys`
* `actor_index_drain`
* `actor_sorted_index_insert`
* `actor_sorted_index_remove`
* `actor_sorted_index_scan`

Key Value Stores are a special type of object though which may be read/written to as long as one has
a reference to that key value store. This is accessible via the system call:
* `key_value_store_open_entry`

## Fields

## Collections