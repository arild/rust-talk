package no.posten.parcelapi.controller

import jakarta.ws.rs.GET
import jakarta.ws.rs.Path
import jakarta.ws.rs.Produces
import jakarta.ws.rs.core.CacheControl
import jakarta.ws.rs.core.MediaType
import jakarta.ws.rs.core.Response
import org.eclipse.microprofile.openapi.annotations.Operation

private const val ONE_MEGABYTE: Long = 1024L * 1024L

@Path("/check")
class HealthResource {

    @GET
    @Path("/status")
    @Produces(MediaType.TEXT_PLAIN)
    @Operation(hidden = true)
    fun status(): String = "parcel-api is on air"

    @GET
    @Produces(MediaType.TEXT_PLAIN)
    @Operation(hidden = true)
    fun health(): Response {
        val runtime = Runtime.getRuntime()
        val total = runtime.totalMemory() / ONE_MEGABYTE
        val free = runtime.freeMemory() / ONE_MEGABYTE
        val active = (runtime.totalMemory() - runtime.freeMemory()) / ONE_MEGABYTE
        val body = """
            parcel-api-quarkus

            Memory:
            total: ${total}mb, free: ${free}mb, active: ${active}mb

            Version:
            quarkus-port
        """.trimIndent()

        val cacheControl = CacheControl()
        cacheControl.isNoCache = true
        return Response.ok(body)
            .type(MediaType.TEXT_PLAIN)
            .cacheControl(cacheControl)
            .build()
    }
}
