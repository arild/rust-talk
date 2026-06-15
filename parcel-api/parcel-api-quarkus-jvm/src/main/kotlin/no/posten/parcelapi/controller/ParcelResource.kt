package no.posten.parcelapi.controller

import io.smallrye.common.annotation.RunOnVirtualThread
import jakarta.inject.Inject
import jakarta.ws.rs.POST
import jakarta.ws.rs.Path
import jakarta.ws.rs.Produces
import jakarta.ws.rs.core.MediaType
import no.posten.parcelapi.controller.response.ParcelResponse
import no.posten.parcelapi.service.StubParcelService

@Path("/v1/parcel")
class ParcelResource @Inject constructor(
    private val parcelService: StubParcelService,
) {

    @POST
    @Produces(MediaType.APPLICATION_JSON)
    @RunOnVirtualThread
    fun getAllParcels(): List<ParcelResponse> =
        parcelService.listParcels().map { it.toResponse() }
}
