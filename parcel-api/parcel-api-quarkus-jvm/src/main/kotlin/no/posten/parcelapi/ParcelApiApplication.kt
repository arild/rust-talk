package no.posten.parcelapi

import io.quarkus.runtime.Quarkus
import io.quarkus.runtime.QuarkusApplication
import io.quarkus.runtime.annotations.QuarkusMain

@QuarkusMain
class ParcelApiApplication : QuarkusApplication {
    override fun run(vararg args: String?): Int {
        Quarkus.waitForExit()
        return 0
    }
}
