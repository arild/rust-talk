package no.posten.parcelapi.controller

import io.swagger.v3.oas.annotations.Hidden
import org.springframework.http.MediaType.TEXT_PLAIN
import org.springframework.http.ResponseEntity
import org.springframework.http.ResponseEntity.ok
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.RestController

private const val oneMegabyte = 1024 * 1024

@Hidden
@RestController
@RequestMapping("check")
class HealthController {

    @GetMapping("status")
    fun status(): ResponseEntity<String> = ok("parcel-api is on air")

    @GetMapping
    fun health(): ResponseEntity<String> =
        ok()
            .contentType(TEXT_PLAIN)
            .body(
                with(Runtime.getRuntime()) {
                    """
                    parcel-api

                    Memory:
                    total: ${totalMemory() / oneMegabyte}mb, free: ${freeMemory() / oneMegabyte}mb, active: ${(totalMemory() - freeMemory()) / oneMegabyte}mb

                    Version:
                    dev
                    """.trimIndent()
                },
            )
}
