fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icon.ico"); // Имя нашего файла иконки
        res.compile().expect("Не удалось скомпилировать иконку");
    }
}