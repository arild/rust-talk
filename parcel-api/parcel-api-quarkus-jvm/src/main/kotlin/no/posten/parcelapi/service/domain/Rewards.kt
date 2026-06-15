package no.posten.parcelapi.service.domain

import java.time.Instant

data class Rewards(
    val rewardsEarnings: List<RewardsEarning>,
)

data class RewardsEarning(
    val validFrom: Instant,
    val validTo: Instant,
    val type: String,
    val text: String,
    val coins: Int,
)
