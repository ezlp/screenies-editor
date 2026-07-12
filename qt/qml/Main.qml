// Main.qml — the 2.0 landing menu. Routes to the SSRP Editor, the Chatlog
// Parser, and the Gallery via a StackView. Phase-0: pages are stubs.
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import com.screenies.app 1.0

ApplicationWindow {
    id: window
    width: 1100
    height: 720
    minimumWidth: 820
    minimumHeight: 560
    visible: true
    title: "ScreeniesEditor 2.0"

    // The Rust backend (CXX-Qt). Proves the bridge is alive via app_version().
    AppBackend { id: backend }

    StackView {
        id: stack
        anchors.fill: parent
        initialItem: menuPage
    }

    // ── Landing menu ──
    Component {
        id: menuPage
        Page {
            ColumnLayout {
                anchors.centerIn: parent
                spacing: 20
                width: 420

                Label {
                    text: "ScreeniesEditor"
                    font.pixelSize: 34
                    font.bold: true
                    Layout.alignment: Qt.AlignHCenter
                }
                Label {
                    text: "Screenshot Roleplay toolkit — SA-MP community"
                    opacity: 0.7
                    Layout.alignment: Qt.AlignHCenter
                }

                MenuTile {
                    title: "SSRP Editor"
                    subtitle: "Crop, chatlog, filters, export"
                    onClicked: stack.push("pages/EditorPage.qml")
                }
                MenuTile {
                    title: "Chatlog Parser"
                    subtitle: "Open a chatlog folder · search in-app"
                    onClicked: stack.push("pages/ChatlogParserPage.qml")
                }
                MenuTile {
                    title: "Gallery"
                    subtitle: "Browse your exported SSRP photos"
                    onClicked: stack.push("pages/GalleryPage.qml")
                }
            }

            Label {
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                anchors.margins: 12
                text: "v" + backend.app_version()
                opacity: 0.5
                font.pixelSize: 12
            }
        }
    }

    // A big clickable menu button. Inline component (Qt 6.3+).
    component MenuTile: Button {
        property string title
        property string subtitle
        Layout.fillWidth: true
        Layout.preferredHeight: 78
        contentItem: ColumnLayout {
            spacing: 2
            Label { text: title; font.pixelSize: 18; font.bold: true }
            Label { text: subtitle; opacity: 0.7; font.pixelSize: 13 }
        }
    }
}
