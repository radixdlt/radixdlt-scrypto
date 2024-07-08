# Caviarnine Adapter

## Calculation of Fees

This section of the document describes how the adapter estimates how much fees have been earned on a liquidity position for the period that it's been opened. There is some amount of knowledge of Caviarnine required to fully understand this calculation which is also described in this section as we go.

First, I will describe some of the basics of Caviarnine that we will build the model on. Caviarnine allows users to concentrate their liquidity in a Uniswap V3 style where users can add liquidity to different bins. A bin is a discrete range in the price range between 0 and infinity. Adding liquidity to a specific bin means that the user's liquidity is only active when the price is inside that bin's range. Once the price exits that of the bin that the user has their assets in then the user stops earning fees. Of course, the user is allowed to add liquidity to multiple different bins so that their liquidity is active in a wider price range.

<p align="center">
  <img width="600px" src="../../diagrams/caviarnine-basic.png" />
</p>

The above diagram depicts the liquidity in some pool that is made up of assets X and Y and the various bins that exist in this pool. The x-axis of the diagram is the bin number. The distance between each bin and the other is called the bin span. In this diagram, the bin span is 10 which we can calculate by subtracting the second bin from the first (27010 - 27000). The bin span is the same throughout the pool and could change for different pools.

The spot price can be calculated from the bin number by using the following equation:

$$
\mathrm{Spot \space Price}(\mathrm{Bin}) = 1.0005 ^ {2\mathrm{Bin} - 54000}
$$

The first bin in the diagram above is 27000 and the last bin starts at 27060 and ends at 27070. With the equation above this means that the diagram has a price range of:

$$
\mathrm{Spot \space Price}(27000) = 1.0005 ^ {54000 - 54000} = 1.0005 ^ 0 = 1
$$

$$
\mathrm{Spot \space Price}(27070) = 1.0005 ^ {54140 - 54000} = 1.0005 ^ {140} = 1.07248941878
$$

The y-axis in the diagram represents the amount of liquidity in each of the bins. To simplify things, this diagram assumes that each of the bins contains an equal amount of liquidity, which is of course not realistic but simplifies things.

The bin where the current price of the asset lies is called the _active bin_. Not all bins hold both the X and Y assets:

* The currently active bin is the only bin that holds both X and Y assets.
* All bins above the active bin hold X assets
* All bins below the active bin hold Y assets

This is seen in the diagram with the two different colors used for the bins and the active bin (the middle one) being split in half containing both of the colors. 

> This bit is not too relevant to the calculation of fees or anything, but is relevant for the overall understanding of the reader on the Caviarnine system. As you've read above, some of the bins only contain one of the assets and not both. Therefore, unlike Uniswap V2 where the users are obligated to provide two assets of equal value to the pool, the users in this case are not obligated to do that and is free to provide a single side of the liquidity or provide both as they see fit.

As such, if a user wishes to contribute liquidity to three bins: the currently active bin, one to the left, and one to the right, then they would contribute X and Y assets to the currently active bin, Y assets only to the lower bin and X assets only to the higher bin. The Caviarnine pool will accept the contribution to the higher and lower bins in full and will only accept contributions to the active bin according to the ratio of assets. This means that there can never be change for contributions to non-active bins but there can be change for contributions to the active bin.

Say that the price has increased from being in bin 27030 as seen in the current diagram to being in 27050 bin, the diagram for that can be seen below:

<p align="center">
  <img width="600px" src="../../diagrams/caviarnine-price-increase.png" />
</p>

The previous active bin, 27030, is no longer made up of both assets; since the price has increased, it is now a lower bin than the current active bin and now contains only the Y asset according to the rules described in this document. Similarly, as 27050 has become the new active bin it is now composed of a mixture of X and Y assets instead of just X. All of the higher bins saw no changes. All of the X assets that used to be in the bins 27030, 27040, and some of the ones in 27050 have been exchanged for Y assets.

> Another important point to note, in Caviarnine we do not own a percentage of the pool. Rather, we own a percentage of the bins we've contributed to.

With the above background information in mind, we can begin to come up with a model for estimating the fees. This will follow a similar model to what we've come up with for Ociswap the will involve calculating the amount we expect to get back based on price action alone. The fees are the difference between the amount we expect to get back based on price action alone and the amount of assets we actually get back when the liquidity position is closed.

Therefore, in simple terms we can say that:

$$
x_{\mathrm{fees}} = x_{\mathrm{actual}} - x_{\mathrm{Price \space Action (calculated)}}
$$

$$
y_{\mathrm{fees}} = y_{\mathrm{actual}} - y_{\mathrm{Price \space Action (calculated)}}
$$

On how to calculate the amount we expect to get back when closing a liquidity position due to price action alone, we do the following:

* When the user opens a liquidity position we record all of the bins that we added liquidity to and the corresponding amount of liquidity added to them.
* When the user closes the liquidity position we calculate the amount of resources we expect to find in each bin based on the pool's current price the amount we expect to find follows the following logic, which is also summarized in the table below:
  * If a bin contained only X resource when the position was first opened and the current price of the pool is below that bin then we expect to get back the same amount of X tokens we initially put in that bin.
  * If a bin contained only Y resource when the position was first opened and the current price of the pool is above that bin then we expect to get back the same amount of Y tokens we initially put in that bin.
  * If a bin contained only X resource when the position was first opened and the current price of the pool is above that bin then we expect that the full amount of X put in that bin was swapped for Y.
  * If a bin contained only Y resource when the position was first opened and the current price of the pool is below that bin then we expect that the full amount of Y put in that bin was swapped for X.
  * If a bin contained only X resource when the position was first opened and the current price of the pool is inside that bin then some amount of the X resources will be swapped for Y (How much?).
  * If a bin contained only Y resource when the position was first opened and the current price of the pool is inside that bin then some amount of the Y resources will be swapped for X (How much?).
  * If a bin contained a composite amount of X and Y resources when the position was first opened and the current price of the pool is above that bin then all of the X resources that were there will be swapped for Y resources.
  * If a bin contained a composite amount of X and Y resources when the position was first opened and the current price of the pool is below that bin then all of the Y resources that were there will be swapped for X resources.
  * If a bin contained a composite amount of X and Y resources when the position was first opened and the current price of the pool is inside that bin then some amount of X or Y resources will be converted into the other resource.


| Original Bin Composition | Current Bin Composition | Expected Amount | 
| ---- | ---- | ---- |
| Entirely X | Entirely X | No Action - Should be same as original | 
| Entirely X | Entirely Y | Full swap of amount X to amount Y | 
| Entirely X | Composite of X and Y | Partial Swap of some X | 
| Entirely Y | Entirely X | Full swap of amount Y to amount X | 
| Entirely Y | Entirely Y | No Action - Should be same as original | 
| Entirely Y | Composite of X and Y | Partial Swap of some Y | 
| Composite of X and Y | Entirely X | Partial Swap of some Y | 
| Composite of X and Y | Entirely Y | Partial Swap of some X | 
| Composite of X and Y | Composite of X and Y | Partial Swap of some X or Y | 


