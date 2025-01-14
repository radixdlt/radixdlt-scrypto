> [!IMPORTANT]
>
> * Please read our [Contributing Guidelines](/CONTRIBUTING.md) before opening a PR.
> * Before creating your PR, please ensure you read the [node branching strategy](https://github.com/radixdlt/babylon-node/blob/main/docs/branching-strategy.md), which is also used in this repository. The end result after completing the merge actions should be that `release/XXX <= develop`, where `XXX` is the latest released protocol version. This ensures that we minimise merge conflicts, and that work doesn't go missing.
> * As per the branching strategy, **you must ensure you select the _correct base branch_**, both for branching from, and in the PR UI above. The following process can be used to decide the base branch:
>   * For README changes or code changes which can wait until the next protocol update to be released, use `develop`. This should be the default for code changes.
>   * For github workflow changes, or code changes which need to go out as a fully-interoperable update to the node at the current protocol version, use `release/XXX`.
>     * Such changes must be tested and reviewed more carefully to mitigate the risk of regression.
>     * Once the change is merged, it is the merger's responsibility to ensure `release/XXX` is merged into the `develop` branch.
> 
> _Please remove this section once you confirm you follow its guidance._

## Summary
<!--
> [!TIP]
> 
> Start with the context of your PR. Why are you making this change? What does it address? Link back to an issue if relevant.
> 
> Then summarise the changes that were made. Bullet points are fine. Feel free to add additional subheadings (using ###) with more information if required.
-->

## Testing
<!--
> [!TIP]
> 
> Explain what testing / verification is done, including manual testing or automated testing.
-->

## Changelog
<!--
> [!TIP]
>
> If the change in your PR is a new feature, or could affect or break any users/integrators, including scrypto developers, dApp developers, transaction creators, or internal integrators, then it likely will need an update to the CHANGELOG.md file.
>
> Changelog entries should include a link to the PR like: [#2053](https://github.com/radixdlt/radixdlt-scrypto/pull/2053) so may need to be added after the PR is created.
>
> After making any required updates, write either of these two:
> * "The changelog has been updated to capture XX changes which affect XX"
> * "The changelog was not updated because this change has no user-facing impact"
-->
