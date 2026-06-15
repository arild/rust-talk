package no.posten.parcelapi.controller

import no.posten.parcelapi.controller.response.ParcelResponse
import no.posten.parcelapi.service.StubParcelService
import org.springframework.http.MediaType.APPLICATION_JSON_VALUE
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.RestController

@RestController
@RequestMapping("v1/parcel")
class ParcelController(private val parcelService: StubParcelService) {

    @PostMapping(produces = [APPLICATION_JSON_VALUE])
    fun getAllParcels(): List<ParcelResponse> = parcelService.listParcels().map { it.toResponse() }
}
