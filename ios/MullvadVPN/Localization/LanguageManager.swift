//
//  LanguageManager.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-06-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Combine
import Foundation
import SwiftUI

class LanguageManager: ObservableObject {
    @Published var selectedLocale = Locale.current
    @Published var layoutDirection: LayoutDirection = Locale
        .characterDirection(forLanguage: Locale.current.languageCode ?? "en") == .rightToLeft ? .rightToLeft :
        .leftToRight

    func setLanguage(_ languageCode: String) {
        selectedLocale = Locale(identifier: languageCode)
    }
}
