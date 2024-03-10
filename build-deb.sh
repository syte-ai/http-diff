PACKAGE_NAME=$(cat Cargo.toml | grep name | awk -F '"' '{print $2}')
PACKAGE_DESCRIPTION=$(cat Cargo.toml | grep description | awk -F '"' '{print $2}')
PACKAGE_VERSION=$(cat Cargo.toml | grep version | head -n 1 | awk -F '"' '{print $2}')
PACKAGE_MAINTAINER=$(cat Cargo.toml | grep authors | head -n 1 | awk -F '"' '{print $2}')

echo "starting building .deb for - $PACKAGE_NAME-$PACKAGE_VERSION"

rm -rf $BASE_DEB_PACKAGE_PATH

BASE_DEB_PACKAGE_PATH=target/deb/$PACKAGE_NAME
PATH_TO_DEBIAN_DIR=$BASE_DEB_PACKAGE_PATH/DEBIAN
PATH_TO_BIN_DIR=$BASE_DEB_PACKAGE_PATH/usr/bin
PATH_TO_APPLICATIONS_DIR=$BASE_DEB_PACKAGE_PATH/usr/share/applications
PATH_TO_MAN_DIR=$BASE_DEB_PACKAGE_PATH/usr/share/man/man1
PATH_TO_ICONS_DIR=$BASE_DEB_PACKAGE_PATH/usr/share/pixmaps

mkdir -p $PATH_TO_DEBIAN_DIR
mkdir -p $PATH_TO_BIN_DIR
mkdir -p $PATH_TO_APPLICATIONS_DIR
mkdir -p $PATH_TO_MAN_DIR
mkdir -p $PATH_TO_ICONS_DIR

cat > $BASE_DEB_PACKAGE_PATH/DEBIAN/control << EOF
Package: $PACKAGE_NAME
Version: $PACKAGE_VERSION
Architecture: amd64
Maintainer: $PACKAGE_MAINTAINER
Description: $PACKAGE_DESCRIPTION
EOF

cat > $PATH_TO_APPLICATIONS_DIR/$PACKAGE_NAME.desktop << EOF
[Desktop Entry]
Name=$PACKAGE_NAME
Comment=$PACKAGE_DESCRIPTION
Exec=/usr/bin/$PACKAGE_NAME
Icon=$PACKAGE_NAME
Terminal=true
Type=Application
Encoding=UTF-8
Categories=Network;Application;
Name[en_US]=$PACKAGE_NAME
EOF

echo 'creating a release build'
cargo build -r

cp target/assets/$PACKAGE_NAME.1 $PATH_TO_MAN_DIR
cp target/release/$PACKAGE_NAME $PATH_TO_BIN_DIR

cp assets/logo.png $PATH_TO_ICONS_DIR/$PACKAGE_NAME.png

chmod 755 $PATH_TO_BIN_DIR/$PACKAGE_NAME
sudo chown root:root $PATH_TO_BIN_DIR/$PACKAGE_NAME

dpkg-deb --build $BASE_DEB_PACKAGE_PATH