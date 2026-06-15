package no.posten.parcelapi.controller.response

import java.time.Instant

data class RewardsResponse(
    val rewardsEarnings: List<RewardsEarningResponse>,
)

data class RewardsEarningResponse(
    val validFrom: Instant,
    val validTo: Instant,
    val type: String,
    val text: String,
    val coins: Int,
)
