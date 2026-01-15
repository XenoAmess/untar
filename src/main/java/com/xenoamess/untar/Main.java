package com.xenoamess.untar;

import picocli.CommandLine;
import picocli.CommandLine.Command;
import picocli.CommandLine.Option;
import picocli.CommandLine.Parameters;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.concurrent.Callable;
import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;
import org.apache.commons.compress.archivers.zip.ZipArchiveEntry;
import org.apache.commons.compress.archivers.zip.ZipArchiveInputStream;
import org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream;
import org.apache.commons.compress.compressors.xz.XZCompressorInputStream;
import org.apache.commons.compress.compressors.bzip2.BZip2CompressorInputStream;

@Command(name = "untar", description = "解压 tar/tar.gz/tgz/tar.xz/tar.bz2/tar.zip 包")
public class Main implements Callable<Integer> {

    @Option(names = {"-C", "--directory"}, description = "解压到指定目录", paramLabel = "DIR")
    private String directory = ".";

    @Option(names = {"-v", "--verbose"}, description = "显示详细信息")
    private boolean verbose = false;

    @Option(names = {"-h", "--help"}, usageHelp = true, description = "显示帮助信息")
    private boolean help = false;

    @Parameters(paramLabel = "FILE", description = "要解压的文件")
    private String archiveFile;

    @Override
    public Integer call() throws Exception {
        File file = new File(archiveFile);
        if (!file.exists()) {
            System.err.println("错误: 文件不存在: " + archiveFile);
            return 1;
        }

        Path destDir = Paths.get(directory);
        Files.createDirectories(destDir);

        String fileName = file.getName().toLowerCase();
        try {
            if (fileName.endsWith(".tar.gz") || fileName.endsWith(".tgz")) {
                extractTarGz(file, destDir);
            } else if (fileName.endsWith(".tar.xz")) {
                extractTarXz(file, destDir);
            } else if (fileName.endsWith(".tar.bz2")) {
                extractTarBz2(file, destDir);
            } else if (fileName.endsWith(".zip")) {
                extractZip(file, destDir);
            } else if (fileName.endsWith(".tar")) {
                extractTar(file, destDir);
            } else {
                System.err.println("错误: 不支持的文件格式: " + fileName);
                return 1;
            }
            if (verbose) {
                System.out.println("解压完成: " + file.getName());
            }
        } catch (IOException e) {
            System.err.println("解压失败: " + e.getMessage());
            return 1;
        }
        return 0;
    }

    private void extractTarGz(File file, Path destDir) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             GzipCompressorInputStream gzis = new GzipCompressorInputStream(fis);
             TarArchiveInputStream tis = new TarArchiveInputStream(gzis)) {
            extractTar(tis, destDir);
        }
    }

    private void extractTarXz(File file, Path destDir) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             XZCompressorInputStream xzis = new XZCompressorInputStream(fis);
             TarArchiveInputStream tis = new TarArchiveInputStream(xzis)) {
            extractTar(tis, destDir);
        }
    }

    private void extractTarBz2(File file, Path destDir) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             BZip2CompressorInputStream bzis = new BZip2CompressorInputStream(fis);
             TarArchiveInputStream tis = new TarArchiveInputStream(bzis)) {
            extractTar(tis, destDir);
        }
    }

    private void extractTar(File file, Path destDir) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             TarArchiveInputStream tis = new TarArchiveInputStream(fis)) {
            extractTar(tis, destDir);
        }
    }

    private void extractTar(TarArchiveInputStream tis, Path destDir) throws IOException {
        TarArchiveEntry entry;
        while ((entry = tis.getNextTarEntry()) != null) {
            Path entryPath = destDir.resolve(entry.getName());
            if (entry.isDirectory()) {
                Files.createDirectories(entryPath);
            } else {
                Files.createDirectories(entryPath.getParent());
                try (FileOutputStream fos = new FileOutputStream(entryPath.toFile())) {
                    byte[] buffer = new byte[8192];
                    int len;
                    while ((len = tis.read(buffer)) > 0) {
                        fos.write(buffer, 0, len);
                    }
                }
                // 保留文件权限
                if (entry.getMode() > 0) {
                    entryPath.toFile().setExecutable((entry.getMode() & 0100) != 0);
                    entryPath.toFile().setWritable((entry.getMode() & 0200) != 0);
                    entryPath.toFile().setReadable((entry.getMode() & 0400) != 0);
                }
            }
            if (verbose) {
                System.out.println("解压: " + entry.getName());
            }
        }
    }

    private void extractZip(File file, Path destDir) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             ZipArchiveInputStream zis = new ZipArchiveInputStream(fis)) {
            ZipArchiveEntry entry;
            while ((entry = zis.getNextEntry()) != null) {
                Path entryPath = destDir.resolve(entry.getName());
                if (entry.isDirectory()) {
                    Files.createDirectories(entryPath);
                } else {
                    Files.createDirectories(entryPath.getParent());
                    try (FileOutputStream fos = new FileOutputStream(entryPath.toFile())) {
                        byte[] buffer = new byte[8192];
                        int len;
                        while ((len = zis.read(buffer)) > 0) {
                            fos.write(buffer, 0, len);
                        }
                    }
                }
                if (verbose) {
                    System.out.println("解压: " + entry.getName());
                }
            }
        }
    }

    public static void main(String[] args) {
        int exitCode = new CommandLine(new Main()).execute(args);
        System.exit(exitCode);
    }
}
