package no.posten.parcelapi.service

import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule
import com.fasterxml.jackson.module.kotlin.KotlinModule
import no.posten.parcelapi.service.domain.Parcel
import org.springframework.beans.factory.annotation.Value
import org.springframework.stereotype.Service
import java.nio.file.Files
import java.nio.file.Paths
import java.time.Instant
import kotlin.io.path.extension
import kotlin.io.path.readText

@Service
class StubParcelService(
    @Value("\${parcel.data.dir}") private val parcelDataDir: String,
) {

    private val objectMapper: ObjectMapper = ObjectMapper()
        .registerModule(KotlinModule.Builder().build())
        .registerModule(JavaTimeModule())

    // Raw JSON contents, one per parcel. Loaded once at startup;
    // every request re-deserialises the strings, so the parse cost lands on the
    // request path (closer to a real service that decodes data from a downstream).
    private val parcelJsons: List<String> = loadParcelJsons()

    fun listParcels(): List<Parcel> {
        val now = Instant.now()
        return parcelJsons
            .shuffled()
            .map { objectMapper.readValue(it, Parcel::class.java) }
            .map { it.copy(features = computeFeatures(it, now)) }
    }

    private fun loadParcelJsons(): List<String> {
        val dir = Paths.get(parcelDataDir)
        require(Files.isDirectory(dir)) { "parcel.data.dir does not exist or is not a directory: $dir" }
        return Files.list(dir).use { stream ->
            stream
                .filter { it.extension == "json" }
                .sorted()
                .toList()
                .map { it.readText() }
        }
    }
}
