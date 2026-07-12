// GalleryPage.qml — STUB. Phase 4: browse exported SSRP photos from a folder
// via bridge/gallery.rs (thumbnail grid, open-in-editor).
import QtQuick
import QtQuick.Controls

Page {
    header: ToolBar {
        Row {
            anchors.fill: parent
            ToolButton { text: "← Menu"; onClicked: stack.pop() }
            Label { text: "Gallery"; anchors.verticalCenter: parent.verticalCenter; leftPadding: 8 }
        }
    }
    Label {
        anchors.centerIn: parent
        horizontalAlignment: Text.AlignHCenter
        text: "Gallery — coming in phase 4.\nThumbnails of your exported photos."
        opacity: 0.7
    }
}
