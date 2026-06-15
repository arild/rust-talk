package no.posten.parcelapi

import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.builder.SpringApplicationBuilder

val environment: String = (System.getenv("SPRING_PROFILES_ACTIVE") ?: "dev")

@SpringBootApplication
class ParcelApiApplication

fun main(args: Array<String>) {
    SpringApplicationBuilder(ParcelApiApplication::class.java)
        .profiles(environment)
        .build()
        .run(*args)
}
