import SwiftUI

struct MullvadPrimaryTextField: View {
    let label: String
    let placeholder: String
    @Binding var text: String
    @State var suggestion: String?
    var isValid: Bool = true

    @FocusState private var isFocused: Bool
    @Environment(\.isEnabled) private var isEnabled

    private var showSuggestion: Bool {
        if let suggestion,
           !suggestion.isEmpty,
           suggestion != text,
           isEnabled {
            return true
        }
        return false
    }

    var body: some View {
        VStack(alignment: .leading) {
            Text(label)
                .foregroundColor(.MullvadTextField.label)
            VStack(spacing: 0) {
                HStack(spacing: 4) {
                    TextField(
                        placeholder,
                        text: $text,
                        prompt: Text(placeholder)
                            .foregroundColor(
                                isEnabled ? .MullvadTextField.inputPlaceholder : .MullvadTextField.textDisabled
                            )
                    )
                    .focused($isFocused)
                    .padding(.vertical, 12)
                    if !text.isEmpty && isEnabled {
                        Button {
                            withAnimation {
                                text = ""
                            }
                        } label: {
                            Image.mullvadIconCross
                        }
                        .padding(0)
                    }
                }
                .zIndex(1)
                .padding(.horizontal, 8)
                .background(
                    isEnabled ? Color.MullvadTextField.background : Color.MullvadTextField
                        .backgroundDisabled)
                .foregroundColor(isEnabled ? .MullvadTextField.textInput : .MullvadTextField.textDisabled)
                .overlay {
                    if isFocused {
                        RoundedCorner(cornerRadius: 4,
                                      corners: !showSuggestion ? [.allCorners] : [
                                          .topLeft,
                                          .topRight
                                      ])
                                      .stroke(
                                          isValid ? Color.MullvadTextField.borderFocused : Color.MullvadTextField.borderError,
                                          lineWidth: 4
                                      )
                    } else if isEnabled {
                        RoundedCorner(cornerRadius: 4,
                                      corners: !showSuggestion ? [.allCorners] : [
                                          .topLeft,
                                          .topRight
                                      ])
                                      .stroke(isValid ? Color.MullvadTextField.border : Color.MullvadTextField.borderError,
                                              lineWidth: 2)
                    }
                }
                .clipShape(RoundedCorner(cornerRadius: 4,
                                         corners: !showSuggestion ? [.allCorners] : [
                                             .topLeft,
                                             .topRight
                                         ]))

                if showSuggestion,
                   let suggestion {
                    HStack {
                        Button {
                            withAnimation {
                                text = suggestion
                            }
                        } label: {
                            Text(suggestion)
                                .foregroundColor(.MullvadTextField.textInput)
                            Spacer()
                        }
                        Button {
                            withAnimation {
                                self.suggestion = nil
                            }
                        } label: {
                            Image.mullvadIconCross
                        }
                    }
                    .transition(.move(edge: .top))
                    .padding(.horizontal, 8)
                    .padding(.vertical, 12)
                    .background(Color.MullvadTextField.backgroundSuggestion)
                }
            }
            .clipShape(
                RoundedCorner(cornerRadius: 4)
            )
        }
    }
}

private struct RoundedCorner: Shape {
    var cornerRadius: CGFloat = .infinity
    var corners: UIRectCorner = .allCorners
    var insertBy: CGFloat = 0

    func path(in rect: CGRect) -> Path {
        let insetRect = rect.insetBy(dx: insertBy, dy: insertBy)
        let path = UIBezierPath(
            roundedRect: insetRect,
            byRoundingCorners: corners,
            cornerRadii: CGSize(width: cornerRadius, height: cornerRadius)
        )
        return Path(path.cgPath)
    }
}

#Preview {
    StatefulPreviewWrapper("") { text in
        VStack {
            MullvadPrimaryTextField(
                label: "Label",
                placeholder: "Placeholder text",
                text: text,
                suggestion: "1234"
            )

            MullvadPrimaryTextField(
                label: "Label",
                placeholder: "Placeholder text",
                text: text,
                suggestion: "1234",
                isValid: false
            )

            MullvadPrimaryTextField(
                label: "Label",
                placeholder: "Placeholder text",
                text: text,
                suggestion: "1234"
            )
            .disabled(true)
        }
        .padding()
        .background(Color.red)
    }
}

class UIMullvadPrimaryTextField: UIHostingController<MullvadPrimaryTextField> {
    let label: String
    let placeholder: String
    var text: String = ""
    var onTextChange: ((String) -> Void)?

    init(label: String, placeholder: String) {
        self.label = label
        self.placeholder = placeholder
        super.init(rootView: MullvadPrimaryTextField(
            label: label,
            placeholder: placeholder,
            text: .constant("")
        )
        )
    }

    override func viewDidLoad() {
        rootView = MullvadPrimaryTextField(
            label: label,
            placeholder: placeholder,
            text: .init(get: {
                self.text
            }, set: { newText in
                self.text = newText
                self.onTextChange?(newText)
            })
        )
        view.backgroundColor = .clear
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("Not implemented")
    }
}

struct UIMullvadPrimaryTextFieldRepresentable: UIViewRepresentable {
    func makeUIView(context: Context) -> some UIView {
        UIMullvadPrimaryTextField(label: "Label", placeholder: "Placeholder").view
    }

    func updateUIView(_ uiView: UIViewType, context: Context) {}
}

#Preview {
    UIMullvadPrimaryTextFieldRepresentable()
        .padding()
        .background(Color.red)
}
