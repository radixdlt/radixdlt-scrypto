# AuthZone

To call a protected method, the caller must place these proofs into their [AuthZone](../../../blueprints/resource/auth_zone),
a space dedicated for using proofs for the purpose of authorized method access.

An AuthZone has the following methods:
* `pop`
* `push`
* `create_proof_of_amount`
* `create_proof_of_non_fungibles`
* `create_proof_of_all`
* `drop_proofs`
* `drop_signature_proofs`
* `drop_regular_proofs`
* `drain`
* `assert_access_rule`
