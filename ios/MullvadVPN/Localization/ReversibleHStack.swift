//
//  ReversibleHStack.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-06-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct ReversibleHStack<First: View, Second: View>: View {
    @Environment(\.layoutDirection) private var layoutDirection
    let first: First
    let second: Second

    init(
        @ViewBuilder content: () -> TupleView<(First, Second)>
    ) {
        let tuple = content().value
        self.first = tuple.0
        self.second = tuple.1
    }

    var body: some View {
        HStack(alignment: .firstTextBaseline) {
            if layoutDirection == .rightToLeft {
                second
                first
            } else {
                first
                second
            }
        }
        .frame(maxWidth: .infinity, alignment: layoutDirection == .rightToLeft ? .trailing : .leading)
    }
}
