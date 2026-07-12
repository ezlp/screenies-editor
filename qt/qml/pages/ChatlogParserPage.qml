// ChatlogParserPage.qml — STUB. Phase 3: open a chatlog folder, index it via
// bridge/chatlog_library.rs, and search/hover the results here.
import QtQuick
import QtQuick.Controls

Page {
    header: ToolBar {
        Row {
            anchors.fill: parent
            ToolButton { text: "← Menu"; onClicked: stack.pop() }
            Label { text: "Chatlog Parser"; anchors.verticalCenter: parent.verticalCenter; leftPadding: 8 }
        }
    }
    Label {
        anchors.centerIn: parent
        horizontalAlignment: Text.AlignHCenter
        text: "Chatlog Parser — coming in phase 3.\nOpen a folder of logs · full-text search in-app."
        opacity: 0.7
    }
}
