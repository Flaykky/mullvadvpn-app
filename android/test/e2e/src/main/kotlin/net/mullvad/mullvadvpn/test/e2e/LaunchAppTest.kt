package net.mullvad.mullvadvpn.test.e2e

import org.junit.jupiter.api.Disabled
import org.junit.jupiter.api.Test

@Disabled
class LaunchAppTest : EndToEndTest() {
    @Test
    fun testLaunchApp() {
        app.launch()
    }
}
