//
//  TermsOfServiceCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 29/01/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Routing
import UIKit

class TermsOfServiceCoordinator: Coordinator, Presenting {
    private let navigationController: RootContainerViewController

    var presentationContext: UIViewController {
        navigationController
    }

    var didFinish: ((TermsOfServiceCoordinator) -> Void)?

    init(navigationController: RootContainerViewController) {
        self.navigationController = navigationController
    }

    func start() {
        let controller = TermsOfServiceViewController()

        controller.showPrivacyPolicy = { [weak self] in
            self?.presentChild(SafariCoordinator(url: ApplicationConfiguration.privacyPolicyURL), animated: true)
        }

        controller.completionHandler = { [weak self] in
            guard let self else { return }
            didFinish?(self)
        }

        navigationController.pushViewController(controller, animated: false)
    }
}
