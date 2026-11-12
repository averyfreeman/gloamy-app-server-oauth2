# Ứng dụng macOS: vận hành, bảo mật và khôi phục ghép nối

Xác minh lần cuối: **11 tháng 11, 2026**.

Ứng dụng Tauri dùng lại giao diện Vue hiện có. Ứng dụng chỉ khởi chạy daemon Gloamy đi kèm khi không tìm thấy daemon cục bộ đang hoạt động. Nếu daemon đang hoạt động, ứng dụng kết nối nhưng không nhận quyền quản lý. WebView chỉ kết nối tới `127.0.0.1` hoặc `::1`; POC này chưa hỗ trợ địa chỉ bind không phải loopback cụ thể.

## Vòng đời daemon

Tùy chọn “Keep daemon running after closing” được lưu trong cài đặt ứng dụng và mặc định tắt. Khi tắt, ứng dụng chỉ yêu cầu dừng êm daemon có dấu hiệu được ứng dụng desktop quản lý. Khi bật, daemon đó tiếp tục phục vụ giao diện trình duyệt. Ứng dụng không dừng daemon không được quản lý khi đóng, kể cả daemon cũ được phát hiện qua HTTP API.

## Khôi phục ghép nối

Trong màn hình Xác thực hoặc Cài đặt, yêu cầu mã dùng một lần mới rồi hoàn tất qua luồng `/pair` hiện có. Mã được truyền qua Unix socket chỉ dành cho cùng người dùng, chỉ giữ trong bộ nhớ và không ghi vào cấu hình hoặc nhật ký. Ghép nối mới thêm hash token mà không thu hồi các thiết bị hiện có. Nếu ghép nối bị tắt, yêu cầu mã sẽ bị từ chối. Không có HTTP endpoint để tạo mã.

## Điều khiển cục bộ và bảo mật

Socket nằm tại `$TMPDIR/gloamy-<uid>/control.sock` (hoặc thư mục tạm tương ứng của hệ điều hành). Thư mục có quyền `0700`, socket có quyền `0600`. Giao thức chỉ có ba thao tác: xem trạng thái, yêu cầu mã ghép nối và dừng êm. Không có lệnh shell tổng quát hoặc thao tác ghi cấu hình. Socket ngăn người dùng cục bộ khác truy cập, nhưng không ngăn tiến trình chạy cùng UID. Xác thực bearer token hiện tại vẫn được giữ nguyên.

## Khắc phục sự cố

- **Địa chỉ đang được sử dụng:** dừng dịch vụ đang chiếm host và port đã cấu hình, hoặc sửa `gateway.host` / `gateway.port` trong `~/.gloamy/config.toml`. Ứng dụng không khởi chạy daemon trùng lặp.
- **Không khôi phục được ghép nối:** kiểm tra ghép nối có được bật không và yêu cầu mã trong ứng dụng desktop. Trình duyệt không thể truy cập Unix socket cục bộ.
- **Không kết nối được daemon hiện có:** kiểm tra daemon hoạt động tại địa chỉ loopback đã cấu hình. Daemon không được quản lý vẫn tiếp tục chạy sau khi đóng ứng dụng.

Xem đầy đủ tại [Bảo mật socket điều khiển desktop](../../security/desktop-control-socket.md).
