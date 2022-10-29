import com.serotonin.modbus4j.ModbusFactory
import com.serotonin.modbus4j.code.DataType
import com.serotonin.modbus4j.ip.IpParameters
import com.serotonin.modbus4j.locator.BaseLocator

data class Register(
    val address: Int,
    val quantity: Int,
    val name: String,
    val scale: Double,
    val frequency: Double,
    val measurement: String,
    val type: String,
)

fun main(args: Array<String>) {
//    val logger = Logger.getLogger("Main.kt")
//    val json =
//        Register::class.java.getResource("registers.json")?.let { String(Files.readAllBytes(Path.of(it.toURI()))) }
//            ?: throw IllegalArgumentException("Failed to open json file");


//    val registers = Gson().fromJson(json, Array<Register>::class.java).toMutableList().sortedBy { it.frequency }

    val master = ModbusFactory().createTcpMaster(IpParameters().apply { host = "192.168.200.1"; port = 6607 }, true)
        .apply { timeout = 8000; retries = 2; init() }
//    val master = ModbusTcpMaster(
//        ModbusTcpMasterCfalseonfig.Builder("192.168.200.1").setPort(6607).setTimeout(Duration.ofMillis(500)).build()
//    )
    println("Connect: ${master.isConnected}, init: ${master.isInitialized}")
    println("Request")
//        val res = master.send(ReadHoldingRegistersRequest(0, 32017, 1)) ?: continue
    val res = master.getValue(
        BaseLocator.holdingRegister(0, 32017, DataType.TWO_BYTE_INT_SIGNED)
    )
//            NumericLocator(0, RegisterRange.HOLDING_REGISTER, 2017, DataType.TWO_BYTE_INT_SIGNED))
    println("Got $res")
//        if (res.isException) {
//            println("Got exception: ${res.exceptionMessage}")
//        } else {
//            val ress = res as ReadHoldingRegistersResponse
//            println("Got data: ${ress.data}")
//        }
    master.destroy();
}