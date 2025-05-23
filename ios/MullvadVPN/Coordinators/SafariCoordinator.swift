//
//  SafariCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 29/03/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Routing
import SafariServices

@MainActor
class SafariCoordinator: Coordinator, Presentable, @preconcurrency SFSafariViewControllerDelegate {
    nonisolated(unsafe) var didFinish: (@Sendable () -> Void)?

    var presentedViewController: UIViewController {
        safariController
    }

    private let safariController: SFSafariViewController

    init(url: URL) {
        safariController = SFSafariViewController(url: url)
        super.init()

        safariController.delegate = self
    }

    func safariViewControllerDidFinish(_ controller: SFSafariViewController) {
        dismiss(animated: true) {
            self.didFinish?()
        }
    }

    func safariViewControllerWillOpenInBrowser(_ controller: SFSafariViewController) {
        dismiss(animated: false) {
            self.didFinish?()
        }
    }
}
