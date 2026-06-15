package no.posten.parcelapi.controller.exceptions

import jakarta.ws.rs.WebApplicationException
import jakarta.ws.rs.core.MediaType
import jakarta.ws.rs.core.Response
import jakarta.ws.rs.ext.ExceptionMapper
import jakarta.ws.rs.ext.Provider
import org.jboss.logging.Logger

@Provider
class GlobalExceptionMapper : ExceptionMapper<Throwable> {
    private val log: Logger = Logger.getLogger(GlobalExceptionMapper::class.java)

    override fun toResponse(exception: Throwable): Response {
        if (exception is WebApplicationException) {
            return exception.response
        }
        log.error("Uncaught exception: ${exception.message}", exception)
        return Response.status(Response.Status.INTERNAL_SERVER_ERROR)
            .type(MediaType.APPLICATION_JSON)
            .entity(ErrorMessage("Internal server error"))
            .build()
    }
}
