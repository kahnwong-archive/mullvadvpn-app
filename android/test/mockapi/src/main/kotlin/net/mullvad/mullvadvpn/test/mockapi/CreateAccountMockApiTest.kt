package net.mullvad.mullvadvpn.test.mockapi

import net.mullvad.mullvadvpn.test.common.extension.clickAgreeOnPrivacyDisclaimer
import net.mullvad.mullvadvpn.test.common.extension.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove
import net.mullvad.mullvadvpn.test.common.extension.dismissChangelogDialogIfShown
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import org.junit.jupiter.api.Test

class CreateAccountMockApiTest : MockApiTest() {
    @Test
    fun testCreateAccountSuccessful() {
        // Arrange
        val createdAccountNumber = "1234123412341234"
        apiDispatcher.apply {
            expectedAccountNumber = createdAccountNumber
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }
        app.launch(endpoint)

        // Act
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        app.waitForLoginPrompt()
        app.attemptCreateAccount()

        app.dismissStorePasswordPromptIfShown()

        // Assert
        val expectedResult = "1234 1234 1234 1234"
        app.ensureAccountCreated(expectedResult)
    }

    @Test
    fun testCreateAccountFailed() {
        // Arrange
        app.launch(endpoint)

        // Act
        device.clickAgreeOnPrivacyDisclaimer()
        device.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove()
        device.dismissChangelogDialogIfShown()
        app.waitForLoginPrompt()
        app.attemptCreateAccount()

        // Assert
        app.ensureAccountCreationFailed()
    }
}
